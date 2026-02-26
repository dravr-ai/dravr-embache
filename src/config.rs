// ABOUTME: Shared configuration types for CLI-based LLM runners
// ABOUTME: Defines runner types, runner configuration, and environment key parsing
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Default timeout for CLI command execution (120 seconds)
const DEFAULT_TIMEOUT_SECS: u64 = 120;

/// Supported CLI runner types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CliRunnerType {
    /// Claude Code CLI (`claude`)
    ClaudeCode,
    /// Cursor Agent CLI (`cursor-agent`)
    CursorAgent,
    /// `OpenCode` CLI (`opencode`)
    OpenCode,
    /// GitHub Copilot CLI (`copilot`)
    Copilot,
}

impl CliRunnerType {
    /// Binary name used to locate the CLI tool on disk
    #[must_use]
    pub const fn binary_name(&self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude",
            Self::CursorAgent => "cursor-agent",
            Self::OpenCode => "opencode",
            Self::Copilot => "copilot",
        }
    }

    /// Environment variable that can override the binary path
    #[must_use]
    pub const fn env_override_key(&self) -> &'static str {
        match self {
            Self::ClaudeCode => "CLAUDE_CODE_BINARY",
            Self::CursorAgent => "CURSOR_AGENT_BINARY",
            Self::OpenCode => "OPENCODE_BINARY",
            Self::Copilot => "COPILOT_BINARY",
        }
    }
}

impl fmt::Display for CliRunnerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClaudeCode => write!(f, "claude_code"),
            Self::CursorAgent => write!(f, "cursor_agent"),
            Self::OpenCode => write!(f, "opencode"),
            Self::Copilot => write!(f, "copilot"),
        }
    }
}

/// Configuration for a CLI runner instance
#[derive(Debug, Clone)]
pub struct RunnerConfig {
    /// Path to the CLI binary
    pub binary_path: PathBuf,
    /// Model override (provider-specific format)
    pub model: Option<String>,
    /// Maximum time to wait for a CLI command to complete
    pub timeout: Duration,
    /// Additional CLI arguments appended to every invocation
    pub extra_args: Vec<String>,
    /// Environment variable keys passed through to the subprocess
    pub allowed_env_keys: Vec<String>,
    /// Working directory for the subprocess
    pub working_directory: Option<PathBuf>,
}

impl RunnerConfig {
    /// Create a new runner configuration with the given binary path
    #[must_use]
    pub fn new(binary_path: PathBuf) -> Self {
        Self {
            binary_path,
            model: None,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            extra_args: Vec::new(),
            allowed_env_keys: default_allowed_env_keys(),
            working_directory: None,
        }
    }

    /// Set the model to use
    #[must_use]
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set the command timeout
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set extra CLI arguments
    #[must_use]
    pub fn with_extra_args(mut self, args: Vec<String>) -> Self {
        self.extra_args = args;
        self
    }

    /// Set the environment variable keys passed through to the subprocess
    #[must_use]
    pub fn with_allowed_env_keys(mut self, keys: Vec<String>) -> Self {
        self.allowed_env_keys = keys;
        self
    }

    /// Set the working directory for the subprocess
    #[must_use]
    pub fn with_working_directory(mut self, dir: PathBuf) -> Self {
        self.working_directory = Some(dir);
        self
    }
}

/// Default set of environment variable keys safe to pass through to subprocesses
#[must_use]
pub fn default_allowed_env_keys() -> Vec<String> {
    ["HOME", "PATH", "TERM", "USER", "LANG"]
        .iter()
        .map(|k| (*k).to_owned())
        .collect()
}

/// Parse a comma-separated list of environment variable keys
#[must_use]
pub fn parse_env_keys(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

use std::num::ParseIntError;

/// Parse a timeout value from a string (in seconds)
///
/// # Errors
///
/// Returns an error if the string cannot be parsed as a `u64`.
pub fn parse_timeout(input: &str) -> Result<Duration, ParseIntError> {
    input.trim().parse::<u64>().map(Duration::from_secs)
}
