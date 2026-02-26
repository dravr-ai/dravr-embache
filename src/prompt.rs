// ABOUTME: Prompt construction from ChatMessage sequences for CLI invocations
// ABOUTME: Extracts system messages and builds role-prefixed prompt strings
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

use crate::types::{ChatMessage, MessageRole};

/// Build a single prompt string from a slice of chat messages
///
/// Each message is prefixed with its role label (`[system]`, `[user]`,
/// `[assistant]`) followed by the content. Messages are separated by
/// double newlines.
#[must_use]
pub fn build_prompt(messages: &[ChatMessage]) -> String {
    let mut parts: Vec<String> = Vec::with_capacity(messages.len());
    for msg in messages {
        let label = match msg.role {
            MessageRole::System => "[system]",
            MessageRole::User => "[user]",
            MessageRole::Assistant => "[assistant]",
        };
        parts.push(format!("{label}\n{}", msg.content));
    }
    parts.join("\n\n")
}

/// Extract the content of the first system message, if any
#[must_use]
pub fn extract_system_message(messages: &[ChatMessage]) -> Option<&str> {
    messages
        .iter()
        .find(|m| m.role == MessageRole::System)
        .map(|m| m.content.as_str())
}

/// Build a prompt string from non-system messages only
///
/// Useful when the CLI tool accepts a separate `--system-prompt` flag
/// and the system message should not be mixed into the user prompt.
#[must_use]
pub fn build_user_prompt(messages: &[ChatMessage]) -> String {
    let non_system: Vec<&ChatMessage> = messages
        .iter()
        .filter(|m| m.role != MessageRole::System)
        .collect();

    let mut parts: Vec<String> = Vec::with_capacity(non_system.len());
    for msg in &non_system {
        let label = match msg.role {
            MessageRole::User => "[user]",
            MessageRole::Assistant => "[assistant]",
            MessageRole::System => unreachable!(),
        };
        parts.push(format!("{label}\n{}", msg.content));
    }
    parts.join("\n\n")
}
