// ABOUTME: Runner management layer bridging MCP tools to embacle LlmProvider instances
// ABOUTME: Provides factory creation, provider type parsing, and multiplex fan-out engine
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

pub mod factory;
pub mod multiplex;

use embacle::config::CliRunnerType;

/// All provider types supported by embacle, in discovery priority order
pub const ALL_PROVIDERS: &[CliRunnerType] = &[
    CliRunnerType::ClaudeCode,
    CliRunnerType::Copilot,
    CliRunnerType::CursorAgent,
    CliRunnerType::OpenCode,
];

/// Parse a provider name string into a `CliRunnerType`
///
/// Accepts multiple naming conventions: `snake_case`, kebab-case, and
/// short forms to be flexible with MCP client input.
pub fn parse_runner_type(s: &str) -> Option<CliRunnerType> {
    match s.to_lowercase().as_str() {
        "claude_code" | "claude" | "claudecode" => Some(CliRunnerType::ClaudeCode),
        "copilot" => Some(CliRunnerType::Copilot),
        "cursor_agent" | "cursoragent" | "cursor-agent" => Some(CliRunnerType::CursorAgent),
        "opencode" | "open_code" => Some(CliRunnerType::OpenCode),
        _ => None,
    }
}

/// Format the list of valid provider names for error messages
pub const fn valid_provider_names() -> &'static str {
    "claude_code, copilot, cursor_agent, opencode"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_snake_case_variants() {
        assert_eq!(
            parse_runner_type("claude_code"),
            Some(CliRunnerType::ClaudeCode)
        );
        assert_eq!(parse_runner_type("copilot"), Some(CliRunnerType::Copilot));
        assert_eq!(
            parse_runner_type("cursor_agent"),
            Some(CliRunnerType::CursorAgent)
        );
        assert_eq!(parse_runner_type("opencode"), Some(CliRunnerType::OpenCode));
    }

    #[test]
    fn parse_short_forms() {
        assert_eq!(parse_runner_type("claude"), Some(CliRunnerType::ClaudeCode));
        assert_eq!(
            parse_runner_type("cursor-agent"),
            Some(CliRunnerType::CursorAgent)
        );
    }

    #[test]
    fn parse_case_insensitive() {
        assert_eq!(parse_runner_type("COPILOT"), Some(CliRunnerType::Copilot));
        assert_eq!(
            parse_runner_type("Claude_Code"),
            Some(CliRunnerType::ClaudeCode)
        );
    }

    #[test]
    fn parse_unknown_returns_none() {
        assert_eq!(parse_runner_type("gpt4"), None);
        assert_eq!(parse_runner_type(""), None);
    }

    #[test]
    fn all_providers_has_four_entries() {
        assert_eq!(ALL_PROVIDERS.len(), 4);
    }
}
