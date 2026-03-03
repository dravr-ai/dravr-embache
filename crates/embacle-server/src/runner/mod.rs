// ABOUTME: Runner management layer bridging REST handlers to embacle LlmProvider instances
// ABOUTME: Provides factory creation, provider type parsing, and multiplex fan-out engine
//
// SPDX-License-Identifier: Apache-2.0
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
    CliRunnerType::GeminiCli,
    CliRunnerType::CodexCli,
    CliRunnerType::GooseCli,
    CliRunnerType::ClineCli,
    CliRunnerType::ContinueCli,
];

/// Parse a provider name string into a `CliRunnerType`
///
/// Accepts multiple naming conventions: `snake_case`, kebab-case, and
/// short forms for flexible input.
pub fn parse_runner_type(s: &str) -> Option<CliRunnerType> {
    match s.to_lowercase().as_str() {
        "claude_code" | "claude" | "claudecode" => Some(CliRunnerType::ClaudeCode),
        "copilot" => Some(CliRunnerType::Copilot),
        "cursor_agent" | "cursoragent" | "cursor-agent" => Some(CliRunnerType::CursorAgent),
        "opencode" | "open_code" => Some(CliRunnerType::OpenCode),
        "gemini" | "gemini_cli" | "geminicli" | "gemini-cli" => Some(CliRunnerType::GeminiCli),
        "codex" | "codex_cli" | "codexcli" | "codex-cli" => Some(CliRunnerType::CodexCli),
        "goose" | "goose_cli" | "goosecli" | "goose-cli" => Some(CliRunnerType::GooseCli),
        "cline" | "cline_cli" | "clinecli" | "cline-cli" => Some(CliRunnerType::ClineCli),
        "continue" | "continue_cli" | "continuecli" | "continue-cli" | "cn" => {
            Some(CliRunnerType::ContinueCli)
        }
        _ => None,
    }
}

/// Format the list of valid provider names for error messages
pub const fn valid_provider_names() -> &'static str {
    "claude_code, copilot, cursor_agent, opencode, gemini_cli, codex_cli, goose_cli, cline_cli, continue_cli"
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
        assert_eq!(
            parse_runner_type("gemini_cli"),
            Some(CliRunnerType::GeminiCli)
        );
        assert_eq!(
            parse_runner_type("codex_cli"),
            Some(CliRunnerType::CodexCli)
        );
        assert_eq!(
            parse_runner_type("goose_cli"),
            Some(CliRunnerType::GooseCli)
        );
        assert_eq!(
            parse_runner_type("cline_cli"),
            Some(CliRunnerType::ClineCli)
        );
        assert_eq!(
            parse_runner_type("continue_cli"),
            Some(CliRunnerType::ContinueCli)
        );
    }

    #[test]
    fn parse_short_forms() {
        assert_eq!(parse_runner_type("claude"), Some(CliRunnerType::ClaudeCode));
        assert_eq!(
            parse_runner_type("cursor-agent"),
            Some(CliRunnerType::CursorAgent)
        );
        assert_eq!(parse_runner_type("gemini"), Some(CliRunnerType::GeminiCli));
        assert_eq!(parse_runner_type("codex"), Some(CliRunnerType::CodexCli));
        assert_eq!(parse_runner_type("goose"), Some(CliRunnerType::GooseCli));
        assert_eq!(parse_runner_type("cline"), Some(CliRunnerType::ClineCli));
        assert_eq!(
            parse_runner_type("continue"),
            Some(CliRunnerType::ContinueCli)
        );
        assert_eq!(parse_runner_type("cn"), Some(CliRunnerType::ContinueCli));
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
    fn all_providers_has_nine_entries() {
        assert_eq!(ALL_PROVIDERS.len(), 9);
    }
}
