# OrA Rust Backend

Security-first AI orchestration backend built with Rust and Axum.

## Quick Start

### HTTP + WebSocket server
The default configuration targets a local Ollama instance at `http://localhost:11434` and auto-selects an installed model, preferring a coder model when available.

```bash
cargo run -- serve
# Server on http://localhost:8001
```

Running `cargo run` with no subcommand still starts the HTTP server. Use `cargo run -- --help` to inspect all modes.

### MCP stdio mode
Run OrA as an MCP server for desktop and IDE integrations.

```bash
cargo run -- mcp
```

### Optional configuration
You can override the local defaults with environment variables or `~/.ora/config.toml`.

```toml
[llm]
provider = "ollama"
model = "auto"
api_base_url = "http://localhost:11434"
```

Useful environment variables:
- `ORA_LLM_PROVIDER`
- `ORA_MODEL`
- `ORA_API_BASE_URL`
- `ORA_LLM_BASE_URL`
- `ORA_API_KEY`
- `BRAVE_SEARCH_API_KEY`
- `ORA_WEB_SEARCH_BASE_URL`

If `BRAVE_SEARCH_API_KEY` is set, the `web_search` tool uses Brave Search. Otherwise it falls back to live DuckDuckGo search results.

## Project Structure
```text
ora-rust/
├── src/
│   ├── gateway/      # HTTP + WebSocket + MCP gateways
│   ├── kernel/       # Governance, authority, tools, agent loop
│   ├── security/     # Gates, sandboxing, vault, crypto
│   ├── llm/          # Provider clients and model resolution
│   ├── orchestration/# DAG routing
│   └── audit/        # Audit logging
├── docs/             # Plans, handoff docs, research
├── frontend/         # Web UI assets
├── marketing/        # GTM and positioning assets
├── scripts/          # Helper scripts
└── tools/            # MCP helper servers and utilities
```

## API Surface
- `GET /health` - Health and version
- `GET /config` - Effective runtime config snapshot
- `GET /vault/status` - Vault status
- `POST /vault/unlock` - Unlock vault
- `POST /vault/lock` - Lock vault
- `GET /router/models` - Live model catalog for the active provider
- `GET /security/status` - Security gate status
- `GET /authority/current` - Effective authority level
- `POST /authority/escalate` - Simulated authority escalation response
- `GET /approvals` - Pending approvals
- `POST /approvals/:id/approve` - Approve a pending request
- `POST /approvals/:id/reject` - Reject a pending request
- `GET /kernel/metrics` - Runtime CPU and memory usage snapshot
- `POST /kernel/process` - Main prompt-processing entry point
- `POST /chat` - Chat endpoint
- `GET /ws` - WebSocket event stream with task lifecycle and approval updates

## Current Status
- `cargo fmt`, `cargo test`, and `cargo build --release` are expected release gates.
- `/chat` and `/kernel/process` share the same execution path and emit lifecycle events.
- WebSocket task cancellation is implemented through the shared task registry.
- Approval list and approval actions are backed by runtime state instead of placeholder responses.
- The `web_search` tool now uses a real provider path.

## Remaining External Validation
- Run a full Claude Desktop or MCP Inspector end-to-end session against the built binary.
- Decide whether to keep `0.1.0` as the first public tag or bump before release.
