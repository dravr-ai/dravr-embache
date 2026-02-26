// ABOUTME: Configuration for the Copilot SDK provider.
// ABOUTME: Reads environment variables and provides defaults for the SDK client.
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::env;
use std::path::PathBuf;

/// Configuration for the Copilot SDK provider.
#[derive(Debug, Clone)]
pub struct CopilotSdkConfig {
    /// Override path to the copilot CLI binary (default: auto-detect via PATH).
    pub cli_path: Option<PathBuf>,
    /// Default model to use for completions.
    pub model: String,
    /// Whether to use stdio transport (default: true, alternative: TCP).
    pub use_stdio: bool,
    /// GitHub token for authentication (optional, uses stored OAuth by default).
    pub github_token: Option<String>,
}

impl CopilotSdkConfig {
    /// Create configuration from environment variables.
    ///
    /// Environment variables:
    /// - `COPILOT_CLI_PATH` — Override path to copilot binary
    /// - `COPILOT_SDK_MODEL` — Default model (default: `claude-opus-4.6`)
    /// - `COPILOT_SDK_TRANSPORT` — Transport mode: `stdio` (default) or `tcp`
    /// - `COPILOT_GITHUB_TOKEN` / `GH_TOKEN` / `GITHUB_TOKEN` — GitHub auth token
    #[must_use]
    pub fn from_env() -> Self {
        let cli_path = env::var("COPILOT_CLI_PATH").ok().map(PathBuf::from);

        let model = env::var("COPILOT_SDK_MODEL").unwrap_or_else(|_| "claude-opus-4.6".to_owned());

        let use_stdio = env::var("COPILOT_SDK_TRANSPORT")
            .map(|v| v != "tcp")
            .unwrap_or(true);

        let github_token = env::var("COPILOT_GITHUB_TOKEN")
            .or_else(|_| env::var("GH_TOKEN"))
            .or_else(|_| env::var("GITHUB_TOKEN"))
            .ok();

        Self {
            cli_path,
            model,
            use_stdio,
            github_token,
        }
    }
}

impl Default for CopilotSdkConfig {
    fn default() -> Self {
        Self {
            cli_path: None,
            model: "claude-opus-4.6".to_owned(),
            use_stdio: true,
            github_token: None,
        }
    }
}
