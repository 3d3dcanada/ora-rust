# Ora-Rust Next Session Prompt

Use this prompt to continue from the current repo state.

---

You are continuing work on `ora-rust`.

Project path: `/home/wess/ai-workspace/ora-rust`
GitHub repo: `https://github.com/3d3dcanada/ora-rust`

Current verified state:
- `cargo test` passes.
- `cargo build --release` produces `target/release/ora`.
- Default runtime now uses local Ollama with `model = "auto"` and prefers `qwen2.5-coder` when available.
- `/chat` has been verified to return a valid response from the local model.
- `/router/models` now uses live Ollama model discovery.
- `build.sh` and `run.sh` are executable.
- `.gitignore` exists and excludes build artifacts and local secret material.

Files that were materially changed in this stabilization pass:
- `src/gateway/http.rs`
- `src/llm/client.rs`
- `src/llm/providers/local.rs`
- `src/security/gates.rs`
- `src/state.rs`
- `src/config.rs`
- `src/kernel/mod.rs`
- `README.md`
- `docs/AGENT_HANDOFF_2026-03-10.md`
- `.gitignore`

Highest-priority remaining work:
1. Replace the placeholder `web_search` implementation in `src/kernel/tools.rs`.
2. Implement real WebSocket task cancellation.
3. Run a manual MCP client validation against `target/release/ora --mcp-mode`.
4. Continue warning cleanup in legacy agent/kernel modules.
5. Decide whether to keep version `0.1.0` as the initial public baseline or bump/tag after the next feature pass.

Important constraints:
- Preserve the new config-driven chat path. Do not reintroduce a hardcoded model path in the gateway or kernel.
- Preserve live `/router/models` discovery.
- Keep the default local experience working without API keys.
- Read `docs/AGENT_HANDOFF_2026-03-10.md` before making architectural changes.

Suggested validation commands:
```bash
cargo fmt --check
cargo test
cargo build --release
printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}\n' | target/release/ora --mcp-mode
printf '{"message":"Reply with exactly OK."}' | curl -sS -X POST http://127.0.0.1:8001/chat -H 'content-type: application/json' -d @-
```
