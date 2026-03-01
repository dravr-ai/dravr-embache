// ABOUTME: Parses model strings with optional provider prefix into (CliRunnerType, model) pairs
// ABOUTME: Supports "provider:model", "provider", and bare "model" with server default fallback
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

use embacle::config::CliRunnerType;

use crate::runner::parse_runner_type;

/// Resolved provider and model from a model string
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedProvider {
    /// The CLI runner type to use
    pub runner_type: CliRunnerType,
    /// Optional model name (None means use provider default)
    pub model: Option<String>,
}

/// Parse a model string into a provider type and optional model name
///
/// Formats supported:
/// - `"copilot:gpt-4o"` → (Copilot, Some("gpt-4o"))
/// - `"claude:opus"` → (`ClaudeCode`, Some("opus"))
/// - `"copilot"` → (Copilot, None) — use provider default model
/// - `"gpt-4o"` → (`default_provider`, Some("gpt-4o")) — no prefix, use server default
pub fn resolve_model(model_str: &str, default_provider: CliRunnerType) -> ResolvedProvider {
    if let Some((prefix, model)) = model_str.split_once(':') {
        if let Some(runner_type) = parse_runner_type(prefix) {
            return ResolvedProvider {
                runner_type,
                model: if model.is_empty() {
                    None
                } else {
                    Some(model.to_owned())
                },
            };
        }
        // Colon present but prefix not recognized — treat as bare model with default provider
        ResolvedProvider {
            runner_type: default_provider,
            model: Some(model_str.to_owned()),
        }
    } else if let Some(runner_type) = parse_runner_type(model_str) {
        // Exact provider name with no model suffix
        ResolvedProvider {
            runner_type,
            model: None,
        }
    } else {
        // Bare model name — use default provider
        ResolvedProvider {
            runner_type: default_provider,
            model: Some(model_str.to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_provider_with_model() {
        let result = resolve_model("copilot:gpt-4o", CliRunnerType::ClaudeCode);
        assert_eq!(result.runner_type, CliRunnerType::Copilot);
        assert_eq!(result.model.as_deref(), Some("gpt-4o"));
    }

    #[test]
    fn resolve_claude_with_model() {
        let result = resolve_model("claude:opus", CliRunnerType::Copilot);
        assert_eq!(result.runner_type, CliRunnerType::ClaudeCode);
        assert_eq!(result.model.as_deref(), Some("opus"));
    }

    #[test]
    fn resolve_provider_only() {
        let result = resolve_model("copilot", CliRunnerType::ClaudeCode);
        assert_eq!(result.runner_type, CliRunnerType::Copilot);
        assert!(result.model.is_none());
    }

    #[test]
    fn resolve_bare_model_uses_default() {
        let result = resolve_model("gpt-4o", CliRunnerType::Copilot);
        assert_eq!(result.runner_type, CliRunnerType::Copilot);
        assert_eq!(result.model.as_deref(), Some("gpt-4o"));
    }

    #[test]
    fn resolve_provider_with_empty_model() {
        let result = resolve_model("copilot:", CliRunnerType::ClaudeCode);
        assert_eq!(result.runner_type, CliRunnerType::Copilot);
        assert!(result.model.is_none());
    }

    #[test]
    fn resolve_unknown_prefix_as_bare_model() {
        let result = resolve_model("unknown:something", CliRunnerType::Copilot);
        assert_eq!(result.runner_type, CliRunnerType::Copilot);
        assert_eq!(result.model.as_deref(), Some("unknown:something"));
    }

    #[test]
    fn resolve_case_insensitive_provider() {
        let result = resolve_model("CLAUDE:opus", CliRunnerType::Copilot);
        assert_eq!(result.runner_type, CliRunnerType::ClaudeCode);
        assert_eq!(result.model.as_deref(), Some("opus"));
    }

    #[test]
    fn resolve_cursor_agent_variants() {
        for prefix in &["cursor_agent", "cursor-agent", "cursoragent"] {
            let model_str = format!("{prefix}:model");
            let result = resolve_model(&model_str, CliRunnerType::Copilot);
            assert_eq!(result.runner_type, CliRunnerType::CursorAgent);
            assert_eq!(result.model.as_deref(), Some("model"));
        }
    }

    #[test]
    fn resolve_opencode_variants() {
        let result = resolve_model("opencode:latest", CliRunnerType::Copilot);
        assert_eq!(result.runner_type, CliRunnerType::OpenCode);
        assert_eq!(result.model.as_deref(), Some("latest"));

        let result = resolve_model("open_code:latest", CliRunnerType::Copilot);
        assert_eq!(result.runner_type, CliRunnerType::OpenCode);
        assert_eq!(result.model.as_deref(), Some("latest"));
    }
}
