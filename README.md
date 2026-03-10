# OrA Rust Backend

Security-first AI orchestration backend built with Rust and Axum.

## Quick Start

### Default local mode
The default configuration now targets a local Ollama instance at `http://localhost:11434` and auto-selects an installed model, preferring `qwen2.5-coder` when available.

```bash
cargo build
cargo run
# Server on http://localhost:8001
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
- `POST /kernel/process` - Main prompt-processing entry point
- `POST /chat` - Chat endpoint
- `GET /ws` - WebSocket event stream

## Current Status
- `cargo test` passes.
- `cargo build --release` produces `target/release/ora`.
- `/chat` works against the default local Ollama setup.
- `/router/models` is backed by live Ollama model discovery.
- `build.sh` and `run.sh` are executable.

## Known Remaining Work
- `src/kernel/tools.rs` still contains a placeholder `web_search` implementation.
- WebSocket task cancellation is still unimplemented.
- Several non-critical compiler warnings remain in older modules.
- MCP desktop/client validation still needs a manual end-to-end pass.
