# Embache — CLI LLM Runners

[![CI](https://github.com/dravr-ai/dravr-embache/actions/workflows/ci.yml/badge.svg)](https://github.com/dravr-ai/dravr-embache/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE.md)

Standalone Rust library that wraps AI CLI tools as pluggable LLM providers via subprocess execution.

Instead of integrating with LLM APIs directly (which require API keys, SDKs, and managing auth), **Embache** delegates to CLI tools that users already have installed and authenticated — getting model upgrades, auth management, and protocol handling for free.

## Supported CLI Runners

| Runner | Binary | Status | Features |
|--------|--------|--------|----------|
| Claude Code | `claude` | ✅ Production | JSON output, streaming, system prompts, session resume |
| GitHub Copilot | `copilot` | ✅ Production | Text parsing, streaming |
| Cursor Agent | `cursor-agent` | ✅ Production | JSON output, streaming, MCP approval |
| OpenCode | `opencode` | ✅ Production | JSON events, session management |

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
embache = "0.1"
```

Use a runner:

```rust
use std::path::PathBuf;
use embache::{ClaudeCodeRunner, RunnerConfig};
use embache::types::{ChatMessage, ChatRequest, LlmProvider};

#[tokio::main]
async fn main() -> Result<(), embache::types::RunnerError> {
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

## Architecture

```
Your Application
    └── embache (this library)
            ├── ClaudeCodeRunner    → spawns `claude -p "prompt" --output-format json`
            ├── CopilotRunner       → spawns `copilot -p "prompt"`
            ├── CursorAgentRunner   → spawns `cursor-agent -p "prompt" --output-format json`
            └── OpenCodeRunner      → spawns `opencode run "prompt" --format json`
```

Each runner implements the `LlmProvider` trait with:
- **`complete()`** — single-shot completion
- **`complete_stream()`** — streaming completion
- **`health_check()`** — verify CLI is installed and authenticated

## Features

- **Zero API keys** — uses CLI tools' own auth (OAuth, API keys managed by the tool)
- **Auto-discovery** — finds installed CLI binaries via `which`
- **Auth readiness** — non-blocking checks, graceful degradation
- **Capability detection** — probes CLI version and supported features
- **Container isolation** — optional container-based execution for production
- **Subprocess safety** — timeout, output limits, environment sandboxing

## Modules

| Module | Purpose |
|--------|---------|
| `types` | Core types: `LlmProvider` trait, `ChatRequest`, `ChatResponse`, `RunnerError` |
| `config` | Runner types, execution modes, configuration |
| `discovery` | Auto-detect installed CLI binaries |
| `auth` | Readiness checking (is the CLI authenticated?) |
| `compat` | Version compatibility and capability detection |
| `process` | Subprocess spawning with timeout and output limits |
| `sandbox` | Environment variable whitelisting, working directory control |
| `container` | Container-based execution backend |
| `prompt` | Prompt building from chat messages |

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
