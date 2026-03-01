# Changelog

## [0.1.1] — 2026-03-01



## [0.1.0] — 2026-03-01

### Added

- feat: add embacle-server crate with OpenAI-compatible REST API Axum HTTP server with /v1/chat/completions, /v1/models, /health, SSE streaming, multiplex fan-out, bearer auth
- feat: add embacle-mcp binary crate with MCP server Stdio/HTTP transports, 7 tools, JSON-RPC 2.0, multiplex fan-out, README MCP section
- feat: add Timeout error kind, enhanced logging, and stdout capture on failure
- feat: add tool_simulation module for text-based tool calling Text-based tool loop for CLI runners with catalog generation, XML parsing, and async execution
- feat: address analysis weak spots, expand tests, add CI branch triggers Fix doc drift, propagate max_tokens, add gh-auth check, 44 new tests (18→62)
- feat: make available_models() configurable at runtime Change LlmProvider::available_models() return type from &'static [&'static str] to &[String] so models can be determined at runtime. Each runner stores its model list in a Vec<String>. CopilotRunner and CopilotSdkRunner discover models via `gh copilot models` at construction time, falling back to static defaults if the command fails.
- feat: default to claude-opus-4.6 for Copilot SDK, Copilot CLI, and Claude Code runners
- feat: add SDK_TOOL_CALLING capability and as_any() for downcasting Add flag for SDK-managed tool loops, as_any() on LlmProvider trait, re-export ToolHandler types
- feat: add Copilot SDK provider behind copilot-sdk feature flag

### Fixed

- fix: release workflow — use macos-14, build --workspace, reset versions to 0.0.1
- fix: rename remaining capitalized Embache references to Embacle
- fix: rename crate from embache to embacle
- fix: wire env keys to sandbox, guard streaming child lifecycle, remove dead ExecutionMode
- fix: use RunnerConfig::new in doc examples
- fix: correct rust-toolchain.toml format



## [0.1.0] — 2026-03-01

### Added

- feat: add embacle-server crate with OpenAI-compatible REST API Axum HTTP server with /v1/chat/completions, /v1/models, /health, SSE streaming, multiplex fan-out, bearer auth
- feat: add embacle-mcp binary crate with MCP server Stdio/HTTP transports, 7 tools, JSON-RPC 2.0, multiplex fan-out, README MCP section
- feat: add Timeout error kind, enhanced logging, and stdout capture on failure
- feat: add tool_simulation module for text-based tool calling Text-based tool loop for CLI runners with catalog generation, XML parsing, and async execution
- feat: address analysis weak spots, expand tests, add CI branch triggers Fix doc drift, propagate max_tokens, add gh-auth check, 44 new tests (18→62)
- feat: make available_models() configurable at runtime Change LlmProvider::available_models() return type from &'static [&'static str] to &[String] so models can be determined at runtime. Each runner stores its model list in a Vec<String>. CopilotRunner and CopilotSdkRunner discover models via `gh copilot models` at construction time, falling back to static defaults if the command fails.
- feat: default to claude-opus-4.6 for Copilot SDK, Copilot CLI, and Claude Code runners
- feat: add SDK_TOOL_CALLING capability and as_any() for downcasting Add flag for SDK-managed tool loops, as_any() on LlmProvider trait, re-export ToolHandler types
- feat: add Copilot SDK provider behind copilot-sdk feature flag

### Fixed

- fix: rename remaining capitalized Embache references to Embacle
- fix: rename crate from embache to embacle
- fix: wire env keys to sandbox, guard streaming child lifecycle, remove dead ExecutionMode
- fix: use RunnerConfig::new in doc examples
- fix: correct rust-toolchain.toml format


