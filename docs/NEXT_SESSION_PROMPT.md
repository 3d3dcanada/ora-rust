# Ora-Rust Next Session Prompt

Use this prompt to continue from the current repo state.

---

You are continuing work on `ora-rust`.

Project path: `/home/wess/ai-workspace/ora-rust`
GitHub repo: `https://github.com/3d3dcanada/ora-rust`

Current verified state:
- `cargo fmt --check`, `cargo test`, and `cargo build --release` all pass.
- `target/release/ora` serves both explicit runtime modes: `serve` and `mcp`.
- `/chat` and `/kernel/process` share the same execution path.
- WebSocket task cancellation is implemented through shared task state.
- `web_search` is implemented and exposed through MCP `tools/list`.
- MCP `initialize`, `tools/list`, and a live `web_search` tool call have been validated.
- `/health`, `/kernel/metrics`, `/approvals`, and `/chat` were all exercised successfully against the running server.
- Default runtime still uses local Ollama with `model = "auto"` and keeps the no-API-key local path working.

Files that were materially changed in this completion pass:
- `src/main.rs`
- `src/state.rs`
- `src/gateway/http.rs`
- `src/gateway/websocket.rs`
- `src/gateway/tasks.rs`
- `src/gateway/mcp.rs`
- `src/kernel/tools.rs`
- `src/kernel/web_search.rs`
- `src/kernel/mod.rs`
- `src/kernel/agent.rs`
- `src/llm/tools.rs`
- `README.md`
- `docs/AGENT_HANDOFF_2026-03-10.md`

Highest-priority remaining work:
1. Run a manual Claude Desktop or MCP Inspector end-to-end session against `target/release/ora mcp`.
2. Decide whether `0.1.0` should be the first public tag or whether to bump before release.
3. Create the release/tag/publish flow once the manual client validation is complete.

Important constraints:
- Preserve the shared task lifecycle between HTTP and WebSocket flows.
- Preserve the real `web_search` provider path and plain-text result normalization.
- Preserve the explicit CLI modes and legacy `--mcp-mode` compatibility.
- Keep the default local Ollama experience working without API keys.
- Read `docs/AGENT_HANDOFF_2026-03-10.md` before making architectural changes.

Suggested validation commands:
```bash
cargo fmt --check
cargo test
cargo build --release
cargo run -- --help
printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}\n' | cargo run --quiet -- mcp
printf '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}\n' | cargo run --quiet -- mcp
printf '{"message":"Reply with exactly OK."}' | curl --max-time 10 -sS -X POST http://127.0.0.1:8001/chat -H 'content-type: application/json' -d @-
```
