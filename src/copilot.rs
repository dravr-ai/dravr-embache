// ABOUTME: GitHub Copilot CLI runner implementing the `LlmProvider` trait
// ABOUTME: Wraps the `copilot` CLI with plain-text output parsing and streaming support
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::io;
use std::process::Stdio;
use std::str;
use std::time::Duration;

use crate::types::{
    ChatRequest, ChatResponse, ChatStream, LlmCapabilities, LlmProvider, RunnerError, StreamChunk,
};
use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio_stream::wrappers::LinesStream;
use tokio_stream::StreamExt;
use tracing::{debug, warn};

use crate::config::RunnerConfig;
use crate::process::run_cli_command;
use crate::prompt::build_prompt;
use crate::sandbox::{apply_sandbox, build_policy};

/// Maximum output size for a single Copilot CLI invocation (50 MiB)
const MAX_OUTPUT_BYTES: usize = 50 * 1024 * 1024;

/// Health check timeout (10 seconds)
const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(10);

/// Health check output limit (4 KiB)
const HEALTH_CHECK_MAX_OUTPUT: usize = 4096;

/// Default model for Copilot CLI
const DEFAULT_MODEL: &str = "claude-sonnet-4.6";

/// GitHub Copilot CLI runner
///
/// Implements `LlmProvider` by delegating to the `copilot` binary in
/// non-interactive mode (`-p`). Copilot CLI outputs plain text (no JSON
/// structure), so the raw stdout is captured as the response content.
/// System messages are embedded into the user prompt since Copilot CLI
/// has no `--system-prompt` flag.
pub struct CopilotRunner {
    config: RunnerConfig,
    default_model: String,
}

impl CopilotRunner {
    /// Create a new Copilot CLI runner with the given configuration
    #[must_use]
    pub fn new(config: RunnerConfig) -> Self {
        let default_model = config
            .model
            .clone()
            .unwrap_or_else(|| DEFAULT_MODEL.to_owned());
        Self {
            config,
            default_model,
        }
    }

    /// Build the base command with common arguments
    fn build_command(&self, prompt: &str, silent: bool) -> Command {
        let mut cmd = Command::new(&self.config.binary_path);

        // Non-interactive prompt mode
        cmd.args(["-p", prompt]);

        let model = self
            .config
            .model
            .as_deref()
            .unwrap_or_else(|| self.default_model());
        cmd.args(["--model", model]);

        // Required for non-interactive mode
        cmd.arg("--allow-all-tools");

        // Disable MCP servers to force text-based tool catalog usage
        cmd.arg("--disable-builtin-mcps");

        // Prevent reading project AGENTS.md instructions
        cmd.arg("--no-custom-instructions");

        // Autonomous mode — no interactive prompts
        cmd.arg("--no-ask-user");

        // Clean text output
        cmd.arg("--no-color");

        if silent {
            // Output only the agent response (no stats footer)
            cmd.arg("-s");
        }

        for arg in &self.config.extra_args {
            cmd.arg(arg);
        }

        if let Ok(policy) = build_policy(self.config.working_directory.as_deref()) {
            apply_sandbox(&mut cmd, &policy);
        }

        cmd
    }

    /// Parse plain-text output into a `ChatResponse`
    fn parse_response(raw: &[u8]) -> Result<ChatResponse, RunnerError> {
        let content = str::from_utf8(raw)
            .map_err(|e| {
                RunnerError::internal(format!("Copilot CLI output is not valid UTF-8: {e}"))
            })?
            .trim()
            .to_owned();

        Ok(ChatResponse {
            content,
            model: "copilot".to_owned(),
            usage: None,
            finish_reason: Some("stop".to_owned()),
        })
    }
}

#[async_trait]
impl LlmProvider for CopilotRunner {
    fn name(&self) -> &'static str {
        "copilot"
    }

    fn display_name(&self) -> &'static str {
        "GitHub Copilot CLI"
    }

    fn capabilities(&self) -> LlmCapabilities {
        // Copilot CLI has no --system-prompt flag; system messages are
        // embedded into the prompt via build_prompt(). Streaming is
        // supported by reading stdout line by line.
        LlmCapabilities::STREAMING
    }

    fn default_model(&self) -> &str {
        &self.default_model
    }

    fn available_models(&self) -> &'static [&'static str] {
        &[
            "claude-sonnet-4.6",
            "claude-opus-4.6",
            "claude-opus-4.6-fast",
            "claude-sonnet-4.5",
            "claude-haiku-4.5",
            "claude-sonnet-4",
            "gpt-5.2-codex",
            "gpt-5.2",
            "gpt-5.1-codex",
            "gpt-5.1",
            "gpt-5-mini",
            "gpt-4.1",
            "gemini-3-pro-preview",
        ]
    }

    async fn complete(&self, request: &ChatRequest) -> Result<ChatResponse, RunnerError> {
        let prompt = build_prompt(&request.messages);
        let mut cmd = self.build_command(&prompt, true);

        let output = run_cli_command(&mut cmd, self.config.timeout, MAX_OUTPUT_BYTES).await?;

        if output.exit_code != 0 {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RunnerError::external_service(
                "copilot",
                format!("copilot exited with code {}: {stderr}", output.exit_code),
            ));
        }

        Self::parse_response(&output.stdout)
    }

    async fn complete_stream(&self, request: &ChatRequest) -> Result<ChatStream, RunnerError> {
        let prompt = build_prompt(&request.messages);
        let mut cmd = self.build_command(&prompt, true);

        // Enable streaming
        cmd.args(["--stream", "on"]);

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            RunnerError::internal(format!("Failed to spawn copilot for streaming: {e}"))
        })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            RunnerError::internal("Failed to capture copilot stdout for streaming")
        })?;

        let reader = BufReader::new(stdout);
        let lines = LinesStream::new(reader.lines());

        let stream = lines.map(move |line_result: Result<String, io::Error>| {
            let line = line_result
                .map_err(|e| RunnerError::internal(format!("Error reading copilot stream: {e}")))?;

            Ok(StreamChunk {
                delta: line,
                is_final: false,
                finish_reason: None,
            })
        });

        Ok(Box::pin(stream))
    }

    async fn health_check(&self) -> Result<bool, RunnerError> {
        let mut cmd = Command::new(&self.config.binary_path);
        cmd.arg("--version");

        let output =
            run_cli_command(&mut cmd, HEALTH_CHECK_TIMEOUT, HEALTH_CHECK_MAX_OUTPUT).await?;

        if output.exit_code == 0 {
            debug!("Copilot CLI health check passed");
            Ok(true)
        } else {
            warn!(
                exit_code = output.exit_code,
                "Copilot CLI health check failed"
            );
            Ok(false)
        }
    }
}
