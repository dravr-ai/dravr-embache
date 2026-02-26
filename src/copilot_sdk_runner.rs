// ABOUTME: CopilotSdkRunner wraps the copilot-sdk for LLM completions with native tool calling.
// ABOUTME: Manages the copilot --headless server lifecycle via JSON-RPC and implements LlmProvider.
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::sync::Arc;

use async_trait::async_trait;
use copilot_sdk::{
    Client, SessionConfig, SessionEventData, SystemMessageConfig, SystemMessageMode,
    Tool as SdkTool, ToolHandler,
};
use tokio::sync::OnceCell;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tracing::{debug, error, info, warn};

use crate::copilot_sdk_config::CopilotSdkConfig;
use crate::types::{
    ChatRequest, ChatResponse, ChatStream, LlmCapabilities, LlmProvider, MessageRole, RunnerError,
    StreamChunk, TokenUsage,
};

/// Available models for the Copilot SDK provider.
static AVAILABLE_MODELS: &[&str] = &[
    "claude-sonnet-4.6",
    "claude-opus-4.6",
    "gpt-5.2-codex",
    "gpt-5.2",
    "gpt-5.1-codex",
    "gpt-5.1",
    "gpt-5-mini",
    "gpt-4.1",
    "gemini-3-pro-preview",
];

/// GitHub Copilot SDK-based LLM provider.
///
/// Communicates with `copilot --headless` via JSON-RPC (stdio or TCP).
/// Unlike the CLI runner (which spawns a subprocess per request), this provider
/// maintains a persistent server connection with native tool calling support.
pub struct CopilotSdkRunner {
    config: CopilotSdkConfig,
    client: OnceCell<Arc<Client>>,
}

impl CopilotSdkRunner {
    /// Create a new provider from environment configuration.
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            config: CopilotSdkConfig::from_env(),
            client: OnceCell::new(),
        }
    }

    /// Create a new provider with explicit configuration.
    #[must_use]
    pub fn with_config(config: CopilotSdkConfig) -> Self {
        Self {
            config,
            client: OnceCell::new(),
        }
    }

    /// Get or initialize the SDK client (lazy startup).
    async fn get_client(&self) -> Result<&Arc<Client>, RunnerError> {
        self.client
            .get_or_try_init(|| async { self.start_client().await })
            .await
    }

    /// Start the copilot CLI in headless server mode.
    async fn start_client(&self) -> Result<Arc<Client>, RunnerError> {
        info!("Starting Copilot SDK client (headless server)");

        let mut builder = Client::builder()
            .use_stdio(self.config.use_stdio)
            .allow_all_tools(false);

        if let Some(ref cli_path) = self.config.cli_path {
            builder = builder.cli_path(cli_path.clone());
        }

        if let Some(ref token) = self.config.github_token {
            builder = builder.github_token(token.clone());
        }

        let client = builder.build().map_err(|e| {
            RunnerError::internal(format!("Failed to create Copilot SDK client: {e}"))
        })?;

        client.start().await.map_err(|e| {
            RunnerError::internal(format!("Failed to start Copilot headless server: {e}"))
        })?;

        info!("Copilot SDK client started successfully");
        Ok(Arc::new(client))
    }

    /// Build a `SessionConfig` from a chat request.
    fn build_session_config(&self, request: &ChatRequest, tools: Vec<SdkTool>) -> SessionConfig {
        let model = request
            .model
            .as_deref()
            .unwrap_or(&self.config.model)
            .to_owned();

        let system_message = request
            .messages
            .iter()
            .find(|m| m.role == MessageRole::System)
            .map(|m| SystemMessageConfig {
                content: Some(m.content.clone()),
                mode: Some(SystemMessageMode::Replace),
            });

        SessionConfig {
            model: Some(model),
            system_message,
            tools,
            ..Default::default()
        }
    }

    /// Extract the user prompt from the last user message in the request.
    fn extract_user_prompt(request: &ChatRequest) -> String {
        request
            .messages
            .iter()
            .rev()
            .find(|m| m.role == MessageRole::User)
            .map(|m| m.content.clone())
            .unwrap_or_default()
    }

    /// Execute a complete conversation turn with native tool calling.
    ///
    /// The SDK handles the tool call → execute → response cycle internally.
    /// `tool_handler` is called synchronously for each tool invocation by the SDK.
    ///
    /// # Errors
    ///
    /// Returns `RunnerError` if the client fails to start, session creation fails,
    /// message sending fails, or the session encounters an error during execution.
    pub async fn execute_with_tools(
        &self,
        request: &ChatRequest,
        tools: Vec<SdkTool>,
        tool_handler: ToolHandler,
    ) -> Result<SdkToolResponse, RunnerError> {
        let client = self.get_client().await?;
        let session_config = self.build_session_config(request, tools.clone());

        let session = client
            .create_session(session_config)
            .await
            .map_err(|e| RunnerError::internal(format!("Failed to create Copilot session: {e}")))?;

        for tool in &tools {
            session
                .register_tool_with_handler(tool.clone(), Some(tool_handler.clone()))
                .await;
        }

        let prompt = Self::extract_user_prompt(request);
        debug!(
            prompt_len = prompt.len(),
            "Sending prompt to Copilot SDK session"
        );

        let mut events = session.subscribe();

        session.send(&*prompt).await.map_err(|e| {
            RunnerError::internal(format!("Failed to send message to Copilot session: {e}"))
        })?;

        let mut result = self.collect_session_events(&mut events).await?;
        if result.model.is_empty() {
            result.model.clone_from(&self.config.model);
        }
        Ok(result)
    }

    /// Consume session events and accumulate into a response
    async fn collect_session_events(
        &self,
        events: &mut copilot_sdk::EventSubscription,
    ) -> Result<SdkToolResponse, RunnerError> {
        let mut state = EventAccumulator::default();

        while let Ok(event) = events.recv().await {
            match state.process_event(&event.data) {
                EventAction::Continue => {}
                EventAction::Done => {
                    debug!("Session idle — conversation turn complete");
                    break;
                }
                EventAction::Error(msg) => {
                    error!(error = %msg, "Copilot session error");
                    return Err(RunnerError::internal(msg));
                }
            }
        }

        Ok(state.into_response())
    }
}

/// Tracks accumulated state from session events
#[derive(Default)]
struct EventAccumulator {
    content: String,
    tool_calls_count: u32,
    model_used: String,
}

/// Action to take after processing an event
enum EventAction {
    Continue,
    Done,
    Error(String),
}

impl EventAccumulator {
    fn process_event(&mut self, data: &SessionEventData) -> EventAction {
        match data {
            SessionEventData::AssistantMessage(msg) => {
                self.content.clone_from(&msg.content);
                EventAction::Continue
            }
            SessionEventData::AssistantMessageDelta(delta) => {
                self.content.push_str(&delta.delta_content);
                EventAction::Continue
            }
            SessionEventData::ToolExecutionStart(_) => {
                self.tool_calls_count += 1;
                EventAction::Continue
            }
            SessionEventData::SessionModelChange(change) => {
                self.model_used.clone_from(&change.new_model);
                EventAction::Continue
            }
            SessionEventData::SessionIdle(_) => EventAction::Done,
            SessionEventData::SessionError(err) => {
                EventAction::Error(format!("Copilot session error: {}", err.message))
            }
            _ => EventAction::Continue,
        }
    }

    fn into_response(self) -> SdkToolResponse {
        SdkToolResponse {
            content: self.content,
            model: self.model_used,
            tool_calls_count: self.tool_calls_count,
            usage: None,
            finish_reason: Some("stop".to_owned()),
        }
    }
}

/// Response from an SDK conversation turn including tool execution metadata.
#[derive(Debug, Clone)]
pub struct SdkToolResponse {
    /// Final assistant response content.
    pub content: String,
    /// Model that generated the response.
    pub model: String,
    /// Number of tool calls executed during the turn.
    pub tool_calls_count: u32,
    /// Token usage (if available from events).
    pub usage: Option<TokenUsage>,
    /// Finish reason.
    pub finish_reason: Option<String>,
}

#[async_trait]
impl LlmProvider for CopilotSdkRunner {
    fn name(&self) -> &'static str {
        "copilot_sdk"
    }

    fn display_name(&self) -> &'static str {
        "GitHub Copilot (SDK)"
    }

    fn capabilities(&self) -> LlmCapabilities {
        LlmCapabilities::STREAMING | LlmCapabilities::SYSTEM_MESSAGES
    }

    fn default_model(&self) -> &str {
        &self.config.model
    }

    fn available_models(&self) -> &'static [&'static str] {
        AVAILABLE_MODELS
    }

    async fn complete(&self, request: &ChatRequest) -> Result<ChatResponse, RunnerError> {
        let client = self.get_client().await?;
        let session_config = self.build_session_config(request, Vec::new());

        let session = client
            .create_session(session_config)
            .await
            .map_err(|e| RunnerError::internal(format!("Failed to create Copilot session: {e}")))?;

        let prompt = Self::extract_user_prompt(request);
        let mut events = session.subscribe();

        session
            .send(&*prompt)
            .await
            .map_err(|e| RunnerError::internal(format!("Copilot completion failed: {e}")))?;

        let mut content = String::new();
        let mut model_used = self.config.model.clone();

        while let Ok(event) = events.recv().await {
            match &event.data {
                SessionEventData::AssistantMessage(msg) => {
                    debug!(msg_len = msg.content.len(), "SDK event: AssistantMessage");
                    content.clone_from(&msg.content);
                }
                SessionEventData::AssistantMessageDelta(delta) => {
                    debug!(delta_len = delta.delta_content.len(), "SDK event: Delta");
                    content.push_str(&delta.delta_content);
                }
                SessionEventData::SessionModelChange(change) => {
                    debug!(model = %change.new_model, "SDK event: ModelChange");
                    model_used.clone_from(&change.new_model);
                }
                SessionEventData::SessionIdle(_) => {
                    debug!("SDK event: SessionIdle — ending");
                    break;
                }
                SessionEventData::SessionError(err) => {
                    return Err(RunnerError::external_service(
                        "copilot-sdk",
                        format!("Copilot session error: {}", err.message),
                    ));
                }
                _ => {
                    debug!(event_type = ?event.data, "SDK event: other");
                }
            }
        }

        debug!(
            content_len = content.len(),
            model = %model_used,
            content_preview = %content.chars().take(200).collect::<String>(),
            "Copilot SDK complete() response"
        );

        Ok(ChatResponse {
            content,
            model: model_used,
            usage: None,
            finish_reason: Some("stop".to_owned()),
        })
    }

    async fn complete_stream(&self, request: &ChatRequest) -> Result<ChatStream, RunnerError> {
        let client = self.get_client().await?;
        let session_config = self.build_session_config(request, Vec::new());

        let session = client
            .create_session(session_config)
            .await
            .map_err(|e| RunnerError::internal(format!("Failed to create Copilot session: {e}")))?;

        let prompt = Self::extract_user_prompt(request);
        let subscription = session.subscribe();

        session
            .send(&*prompt)
            .await
            .map_err(|e| RunnerError::internal(format!("Copilot streaming failed: {e}")))?;

        let stream = BroadcastStream::new(subscription.receiver).filter_map(move |event_result| {
            match event_result {
                Ok(event) => match &event.data {
                    SessionEventData::AssistantMessageDelta(delta) => Some(Ok(StreamChunk {
                        delta: delta.delta_content.clone(),
                        is_final: false,
                        finish_reason: None,
                    })),
                    SessionEventData::SessionIdle(_) => Some(Ok(StreamChunk {
                        delta: String::new(),
                        is_final: true,
                        finish_reason: Some("stop".to_owned()),
                    })),
                    SessionEventData::SessionError(err) => {
                        Some(Err(RunnerError::external_service(
                            "copilot-sdk",
                            format!("Copilot stream error: {}", err.message),
                        )))
                    }
                    _ => None,
                },
                Err(_) => None,
            }
        });

        Ok(Box::pin(stream))
    }

    async fn health_check(&self) -> Result<bool, RunnerError> {
        let client: &Arc<Client> = match self.get_client().await {
            Ok(c) => c,
            Err(_) => return Ok(false),
        };

        match client.get_status().await {
            Ok(status) => {
                info!(version = %status.version, "Copilot SDK health check OK");
                Ok(true)
            }
            Err(e) => {
                warn!(error = %e, "Copilot SDK health check failed");
                Ok(false)
            }
        }
    }
}

impl Drop for CopilotSdkRunner {
    fn drop(&mut self) {
        if let Some(client) = self.client.get() {
            let client = client.clone();
            tokio::spawn(async move {
                client.stop().await;
            });
        }
    }
}
