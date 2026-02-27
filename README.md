# Embacle — LLM Runners

[![CI](https://github.com/dravr-ai/dravr-embacle/actions/workflows/ci.yml/badge.svg)](https://github.com/dravr-ai/dravr-embacle/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE.md)

Standalone Rust library that wraps AI CLI tools and SDKs as pluggable LLM providers.

Instead of integrating with LLM APIs directly (which require API keys, SDKs, and managing auth), **Embacle** delegates to CLI tools that users already have installed and authenticated — getting model upgrades, auth management, and protocol handling for free. For GitHub Copilot, an optional SDK mode maintains a persistent JSON-RPC connection for native tool calling.

## Supported Runners

### CLI Runners (subprocess-based)

| Runner | Binary | Features |
|--------|--------|----------|
| Claude Code | `claude` | JSON output, streaming, system prompts, session resume |
| GitHub Copilot | `copilot` | Text parsing, streaming |
| Cursor Agent | `cursor-agent` | JSON output, streaming, MCP approval |
| OpenCode | `opencode` | JSON events, session management |

### SDK Runners (persistent connection)

| Runner | Feature Flag | Features |
|--------|-------------|----------|
| GitHub Copilot SDK | `copilot-sdk` | Persistent JSON-RPC via `copilot --headless`, native tool calling, streaming |

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
embacle = { git = "https://github.com/dravr-ai/dravr-embacle.git", branch = "main" }
```

Use a CLI runner:

```rust
use std::path::PathBuf;
use embacle::{ClaudeCodeRunner, RunnerConfig};
use embacle::types::{ChatMessage, ChatRequest, LlmProvider};

#[tokio::main]
async fn main() -> Result<(), embacle::types::RunnerError> {
    let config = RunnerConfig::new(PathBuf::from("claude"));
    let runner = ClaudeCodeRunner::new(config);

    let request = ChatRequest::new(vec![
        ChatMessage::user("What is the capital of France?"),
    ]);

    let response = runner.complete(&request).await?;
    println!("{}", response.content);
    Ok(())
}
```

### Copilot SDK (feature flag)

Enable the `copilot-sdk` feature for persistent JSON-RPC instead of per-request subprocesses:

```toml
[dependencies]
embacle = { git = "https://github.com/dravr-ai/dravr-embacle.git", branch = "main", features = ["copilot-sdk"] }
```

```rust
use embacle::{CopilotSdkRunner, CopilotSdkConfig};
use embacle::types::{ChatMessage, ChatRequest, LlmProvider};

#[tokio::main]
async fn main() -> Result<(), embacle::types::RunnerError> {
    // Reads COPILOT_SDK_MODEL, COPILOT_GITHUB_TOKEN, etc. from env
    let runner = CopilotSdkRunner::from_env();

    let request = ChatRequest::new(vec![
        ChatMessage::user("Explain Rust ownership"),
    ]);

    let response = runner.complete(&request).await?;
    println!("{}", response.content);
    Ok(())
}
```

The SDK runner starts `copilot --headless` once and reuses the connection across requests. Configuration via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `COPILOT_CLI_PATH` | auto-detect | Override path to copilot binary |
| `COPILOT_SDK_MODEL` | `claude-sonnet-4.6` | Default model for completions |
| `COPILOT_SDK_TRANSPORT` | `stdio` | Transport mode: `stdio` or `tcp` |
| `COPILOT_GITHUB_TOKEN` | stored OAuth | GitHub auth token (falls back to `GH_TOKEN`, `GITHUB_TOKEN`) |

## Architecture

```
Your Application
    └── embacle (this library)
            │
            ├── CLI Runners (subprocess per request)
            │   ├── ClaudeCodeRunner    → spawns `claude -p "prompt" --output-format json`
            │   ├── CopilotRunner       → spawns `copilot -p "prompt"`
            │   ├── CursorAgentRunner   → spawns `cursor-agent -p "prompt" --output-format json`
            │   └── OpenCodeRunner      → spawns `opencode run "prompt" --format json`
            │
            ├── SDK Runners (persistent connection, behind feature flag)
            │   └── CopilotSdkRunner    → JSON-RPC to `copilot --headless`
            │
            └── Tool Simulation (text-based tool calling for CLI runners)
                └── execute_with_text_tools()  → catalog injection, XML parsing, tool loop
```

All runners implement the same `LlmProvider` trait:
- **`complete()`** — single-shot completion
- **`complete_stream()`** — streaming completion
- **`health_check()`** — verify the runner is available and authenticated

The `CopilotSdkRunner` additionally provides:
- **`execute_with_tools()`** — native tool calling via the SDK's session and tool handler infrastructure

### Text-Based Tool Calling (CLI runners)

CLI runners don't have native tool calling, so Embacle provides a text-based simulation layer. It injects a tool catalog into the system prompt and parses `<tool_call>` XML blocks from the LLM response, looping until the model stops calling tools.

```rust
use embacle::tool_simulation::{
    FunctionDeclaration, FunctionCall, FunctionResponse,
    execute_with_text_tools,
};
use embacle::types::{ChatMessage, LlmProvider};
use std::sync::Arc;
use serde_json::json;

let declarations = vec![
    FunctionDeclaration {
        name: "get_weather".into(),
        description: "Get current weather for a city".into(),
        parameters: Some(json!({"type": "object", "properties": {"city": {"type": "string"}}})),
    },
];

let handler = Arc::new(|name: &str, args: &serde_json::Value| -> FunctionResponse {
    FunctionResponse {
        name: name.to_owned(),
        response: json!({"temperature": 22, "conditions": "sunny"}),
    }
});

let mut messages = vec![ChatMessage::user("What's the weather in Paris?")];
let result = execute_with_text_tools(
    &runner,          // any LlmProvider (CopilotRunner, ClaudeCodeRunner, etc.)
    &mut messages,
    &declarations,
    handler,
    5,                // max iterations
).await?;

println!("{}", result.content);           // final response with tools stripped
println!("Tool calls: {}", result.tool_calls_count);
```

The pure functions are also available individually for custom loop implementations:

| Function | Purpose |
|----------|---------|
| `generate_tool_catalog()` | Converts declarations into a markdown catalog |
| `inject_tool_catalog()` | Appends catalog to the system message |
| `parse_tool_call_blocks()` | Parses `<tool_call>` XML blocks from response text |
| `strip_tool_call_blocks()` | Returns clean text with tool blocks removed |
| `format_tool_results_as_text()` | Formats results as `<tool_result>` XML blocks |

## Features

- **Zero API keys** — uses CLI tools' own auth (OAuth, API keys managed by the tool)
- **Auto-discovery** — finds installed CLI binaries via `which`
- **Auth readiness** — non-blocking checks, graceful degradation
- **Capability detection** — probes CLI version and supported features
- **Container isolation** — optional container-based execution for production
- **Subprocess safety** — timeout, output limits, environment sandboxing
- **Feature flags** — SDK integrations are opt-in to keep the default dependency footprint minimal

## Modules

| Module | Feature | Purpose |
|--------|---------|---------|
| `types` | default | Core types: `LlmProvider` trait, `ChatRequest`, `ChatResponse`, `RunnerError` |
| `config` | default | Runner types, execution modes, configuration |
| `discovery` | default | Auto-detect installed CLI binaries |
| `auth` | default | Readiness checking (is the CLI authenticated?) |
| `compat` | default | Version compatibility and capability detection |
| `process` | default | Subprocess spawning with timeout and output limits |
| `sandbox` | default | Environment variable whitelisting, working directory control |
| `container` | default | Container-based execution backend |
| `prompt` | default | Prompt building from chat messages |
| `tool_simulation` | default | Text-based tool calling for CLI runners (`<tool_call>` XML protocol) |
| `copilot_sdk_runner` | `copilot-sdk` | Copilot SDK runner (persistent JSON-RPC) |
| `copilot_sdk_config` | `copilot-sdk` | Copilot SDK configuration from environment |
| `tool_bridge` | `copilot-sdk` | Tool definition conversion for native tool calling |

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
