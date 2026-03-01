// ABOUTME: Server state holding default provider and lazily-created runner cache
// ABOUTME: Stateless per-request routing with no mutable active provider or model
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::collections::HashMap;
use std::sync::Arc;

use embacle::config::CliRunnerType;
use embacle::types::{LlmProvider, RunnerError};
use tokio::sync::Mutex;

use crate::runner::factory;

/// Shared server state handle
pub type SharedState = Arc<ServerState>;

/// Server state with immutable default provider and lazy runner cache
///
/// Unlike the MCP server, there is no mutable active provider or model.
/// All provider routing is determined per-request via the model string.
/// The runner cache avoids recreating providers on every request.
pub struct ServerState {
    /// Default provider used when model string has no prefix
    default_provider: CliRunnerType,
    /// Lazily-created runners keyed by provider type
    runners: Mutex<HashMap<CliRunnerType, Arc<dyn LlmProvider>>>,
}

impl ServerState {
    /// Create server state with the given default provider
    pub fn new(default_provider: CliRunnerType) -> Self {
        Self {
            default_provider,
            runners: Mutex::new(HashMap::new()),
        }
    }

    /// Get the server's default provider type
    pub const fn default_provider(&self) -> CliRunnerType {
        self.default_provider
    }

    /// Get or lazily create a runner for the given provider type
    ///
    /// Created runners are cached for future calls.
    pub async fn get_runner(
        &self,
        provider: CliRunnerType,
    ) -> Result<Arc<dyn LlmProvider>, RunnerError> {
        let mut runners = self.runners.lock().await;

        if let Some(runner) = runners.get(&provider) {
            return Ok(Arc::clone(runner));
        }

        let runner = factory::create_runner(provider)?;
        let runner: Arc<dyn LlmProvider> = Arc::from(runner);
        runners.insert(provider, Arc::clone(&runner));
        drop(runners);
        Ok(runner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_provider_is_stored() {
        let state = ServerState::new(CliRunnerType::Copilot);
        assert_eq!(state.default_provider(), CliRunnerType::Copilot);
    }

    #[test]
    fn default_provider_claude() {
        let state = ServerState::new(CliRunnerType::ClaudeCode);
        assert_eq!(state.default_provider(), CliRunnerType::ClaudeCode);
    }
}
