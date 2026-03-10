# Ora-Rust Agent Handoff (2026-03-10)

## Completed
- [x] Release gates pass: `cargo fmt --check`, `cargo test`, and `cargo build --release`.
- [x] `/chat` and `/kernel/process` now share a single task execution path.
- [x] WebSocket task cancellation is implemented through a shared task registry.
- [x] Approval list and approval actions are backed by runtime approval state.
- [x] `web_search` uses a real provider path with provider-specific normalization.
- [x] MCP now advertises `web_search` in `tools/list`.
- [x] Added CLI help with explicit `serve` and `mcp` runtime modes.
- [x] Cleared the remaining dead-code warning in `src/kernel/agent.rs`.
- [x] Updated the README to reflect the current runtime surface.

## Verified Runtime State
- `cargo run -- --help` prints explicit `serve` and `mcp` modes.
- `printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}\n' | cargo run --quiet -- mcp` succeeds.
- `printf '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}\n' | cargo run --quiet -- mcp` includes `web_search`.
- `printf '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"web_search","arguments":{"query":"Rust programming language","num_results":2}}}\n' | cargo run --quiet -- mcp` returns live results.
- `curl -sS http://127.0.0.1:8001/health` returns `{"status":"ok","version":"0.1.0"}`.
- `curl -sS http://127.0.0.1:8001/kernel/metrics` returns live CPU and memory usage.
- `curl -sS http://127.0.0.1:8001/approvals` returns a real approval-state payload.
- `printf '{"message":"Reply with exactly OK."}' | curl --max-time 10 -sS -X POST http://127.0.0.1:8001/chat -H 'content-type: application/json' -d @-` returns `OK`.
- Release binary path: `target/release/ora`.

## Current Repo/Release Notes
- Git is initialized and `origin` is already configured for `main`.
- The repository version remains `0.1.0`.
- Search uses Brave when `BRAVE_SEARCH_API_KEY` is configured and falls back to DuckDuckGo otherwise.

## Remaining External Work
- [ ] Run a manual Claude Desktop or MCP Inspector end-to-end session.
- [ ] Decide whether to keep `0.1.0` as the first public tag or bump before release.
- [ ] Push/tag/release only when you are ready to publish.

## Recommended Next Order
1. Run one manual desktop/IDE MCP session against `target/release/ora mcp`.
2. Decide on the public version tag.
3. Create the release/tag once the manual validation is complete.

## Validation Commands
```bash
cargo fmt --check
cargo test
cargo build --release
cargo run -- --help
printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}\n' | cargo run --quiet -- mcp
printf '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}\n' | cargo run --quiet -- mcp
printf '{"message":"Reply with exactly OK."}' | curl --max-time 10 -sS -X POST http://127.0.0.1:8001/chat -H 'content-type: application/json' -d @-
```
