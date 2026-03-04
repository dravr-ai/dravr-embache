// ABOUTME: `OpenCode` CLI runner implementing the `LlmProvider` trait
// ABOUTME: Wraps the `opencode` CLI with JSON output parsing (no streaming support)
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::str;

use crate::cli_common::{CliRunnerBase, MAX_OUTPUT_BYTES};
use crate::types::{
    ChatRequest, ChatResponse, ChatStream, LlmCapabilities, LlmProvider, RunnerError, TokenUsage,
};
use async_trait::async_trait;
use serde::Deserialize;
use tokio::process::Command;
use tracing::instrument;

use crate::config::RunnerConfig;
use crate::process::run_cli_command;
use crate::prompt::build_prompt;
use crate::sandbox::{apply_sandbox, build_policy};

/// `OpenCode` CLI response JSON structure
#[derive(Debug, Deserialize)]
struct OpenCodeResponse {
    result: Option<String>,
    #[serde(default)]
    is_error: bool,
    session_id: Option<String>,
    usage: Option<OpenCodeUsage>,
}

/// Token usage from `OpenCode` CLI
#[derive(Debug, Deserialize)]
struct OpenCodeUsage {
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
}

/// Default model for `OpenCode`
const DEFAULT_MODEL: &str = "anthropic/claude-sonnet-4";

/// Fallback model list when no runtime override is available
const FALLBACK_MODELS: &[&str] = &[
    "anthropic/claude-sonnet-4",
    "anthropic/claude-opus-4",
    "openai/gpt-5",
];

/// `OpenCode` CLI runner
///
/// Implements `LlmProvider` by delegating to the `opencode` binary with
/// `--format json`. Models use `provider/model` format (e.g.
/// `anthropic/claude-sonnet-4`). Streaming is not supported.
pub struct OpenCodeRunner {
    base: CliRunnerBase,
}

impl OpenCodeRunner {
    /// Create a new `OpenCode` runner with the given configuration
    #[must_use]
    pub fn new(config: RunnerConfig) -> Self {
        Self {
            base: CliRunnerBase::new(config, DEFAULT_MODEL, FALLBACK_MODELS),
        }
    }

    /// Store a session ID for later resumption
    pub async fn set_session(&self, key: &str, session_id: &str) {
        self.base.set_session(key, session_id).await;
    }

    /// Build the base command with common arguments
    fn build_command(&self, prompt: &str) -> Command {
        let mut cmd = Command::new(&self.base.config.binary_path);
        cmd.args(["run", prompt, "--format", "json"]);

        let model = self
            .base
            .config
            .model
            .as_deref()
            .unwrap_or_else(|| self.base.default_model());
        cmd.args(["--model", model]);

        for arg in &self.base.config.extra_args {
            cmd.arg(arg);
        }

        if let Ok(policy) = build_policy(
            self.base.config.working_directory.as_deref(),
            &self.base.config.allowed_env_keys,
        ) {
            apply_sandbox(&mut cmd, &policy);
        }

        cmd
    }

    /// Parse an `OpenCode` JSON response into a `ChatResponse`
    fn parse_response(raw: &[u8]) -> Result<(ChatResponse, Option<String>), RunnerError> {
        let text = str::from_utf8(raw).map_err(|e| {
            RunnerError::internal(format!("OpenCode output is not valid UTF-8: {e}"))
        })?;

        let parsed: OpenCodeResponse = serde_json::from_str(text).map_err(|e| {
            RunnerError::internal(format!("Failed to parse OpenCode JSON response: {e}"))
        })?;

        if parsed.is_error {
            return Err(RunnerError::external_service(
                "opencode",
                parsed
                    .result
                    .as_deref()
                    .unwrap_or("Unknown error from OpenCode"),
            ));
        }

        let content = parsed.result.unwrap_or_default();
        let usage = parsed.usage.map(|u| TokenUsage {
            prompt_tokens: u.input_tokens.unwrap_or(0),
            completion_tokens: u.output_tokens.unwrap_or(0),
            total_tokens: u.input_tokens.unwrap_or(0) + u.output_tokens.unwrap_or(0),
        });

        let response = ChatResponse {
            content,
            model: "opencode".to_owned(),
            usage,
            finish_reason: Some("stop".to_owned()),
            warnings: None,
        };

        Ok((response, parsed.session_id))
    }
}

#[async_trait]
impl LlmProvider for OpenCodeRunner {
    crate::delegate_provider_base!("opencode", "OpenCode CLI", LlmCapabilities::empty());

    #[instrument(skip_all, fields(runner = "opencode"))]
    async fn complete(&self, request: &ChatRequest) -> Result<ChatResponse, RunnerError> {
        let prompt = build_prompt(&request.messages);
        let mut cmd = self.build_command(&prompt);

        if let Some(model) = &request.model {
            if let Some(sid) = self.base.get_session(model).await {
                cmd.args(["--session", &sid]);
            }
        }

        let output = run_cli_command(&mut cmd, self.base.config.timeout, MAX_OUTPUT_BYTES).await?;
        self.base.check_exit_code(&output, "opencode")?;

        let (response, session_id) = Self::parse_response(&output.stdout)?;

        if let Some(sid) = session_id {
            if let Some(model) = &request.model {
                self.base.set_session(model, &sid).await;
            }
        }

        Ok(response)
    }

    #[instrument(skip_all, fields(runner = "opencode"))]
    async fn complete_stream(&self, _request: &ChatRequest) -> Result<ChatStream, RunnerError> {
        Err(RunnerError::internal(
            "OpenCode CLI does not support streaming responses",
        ))
    }
}
