# embacle-server

OpenAI-compatible REST API server that proxies requests to [embacle](https://github.com/dravr-ai/dravr-embacle) LLM runners. Any client that speaks the OpenAI chat completions API can use it without modification.

## Install

```bash
cargo install embacle-server
```

## Usage

```bash
# Start with default provider on localhost:3000
embacle-server

# Specify provider and bind address
embacle-server --provider claude_code --port 8080 --host 0.0.0.0
```

## Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/chat/completions` | Chat completion (streaming and non-streaming) |
| `GET` | `/v1/models` | List available providers and models |
| `GET` | `/health` | Per-provider readiness check |

## Model Routing

The `model` field determines which provider handles the request. Use a `provider:model` prefix to target a specific runner, or pass a bare model name to use the server's default.

```bash
# Explicit provider
curl http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model": "claude:opus", "messages": [{"role": "user", "content": "hello"}]}'

# Default provider
curl http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model": "gpt-4o", "messages": [{"role": "user", "content": "hello"}]}'
```

## Multiplex

Pass an array of models to fan out the same prompt to multiple providers concurrently:

```bash
curl http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model": ["copilot:gpt-4o", "claude:opus"], "messages": [{"role": "user", "content": "hello"}]}'
```

Streaming is not supported for multiplex requests.

## Authentication

Optional. Set `EMBACLE_API_KEY` to require bearer token auth. When unset, all requests are allowed (localhost dev mode).

```bash
EMBACLE_API_KEY=my-secret embacle-server
curl http://localhost:3000/v1/models -H "Authorization: Bearer my-secret"
```

## Requirements

At least one supported CLI tool must be installed and authenticated:
- `claude` (Claude Code)
- `copilot` (GitHub Copilot)
- `cursor-agent` (Cursor Agent)
- `opencode` (OpenCode)

## License

Apache-2.0 — see [LICENSE-APACHE](../../LICENSE-APACHE).

Full project documentation: [github.com/dravr-ai/dravr-embacle](https://github.com/dravr-ai/dravr-embacle)
