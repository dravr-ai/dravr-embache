// ABOUTME: Factory for creating embacle LlmProvider instances from runner type identifiers
// ABOUTME: Resolves binary paths and constructs the appropriate runner with default config
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::env;

use embacle::config::{CliRunnerType, RunnerConfig};
use embacle::discovery::resolve_binary;
use embacle::types::{LlmProvider, RunnerError};
use embacle::{ClaudeCodeRunner, CopilotRunner, CursorAgentRunner, OpenCodeRunner};

/// Create an `LlmProvider` instance for the given runner type
///
/// Resolves the CLI binary via environment variable override or PATH lookup,
/// then constructs the appropriate runner with default configuration.
pub fn create_runner(runner_type: CliRunnerType) -> Result<Box<dyn LlmProvider>, RunnerError> {
    let binary_name = runner_type.binary_name();
    let env_key = runner_type.env_override_key();
    let env_override = env::var(env_key).ok();

    let binary_path = resolve_binary(binary_name, env_override.as_deref())?;
    let config = RunnerConfig::new(binary_path);

    let runner: Box<dyn LlmProvider> = match runner_type {
        CliRunnerType::ClaudeCode => Box::new(ClaudeCodeRunner::new(config)),
        CliRunnerType::Copilot => Box::new(CopilotRunner::new(config)),
        CliRunnerType::CursorAgent => Box::new(CursorAgentRunner::new(config)),
        CliRunnerType::OpenCode => Box::new(OpenCodeRunner::new(config)),
    };

    Ok(runner)
}
