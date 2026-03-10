# Ora-Rust Release Completion Plan (2026-03-10)

## Execution Update
- [x] `cargo fmt` completed successfully.
- [x] `cargo test` now passes.
- [x] `cargo build --release` produces `target/release/ora`.
- [x] `/chat` now uses configuration-driven LLM selection and returns a valid local-model response.
- [x] `/router/models` now uses live Ollama discovery.
- [x] `.gitignore` was added.
- [x] `build.sh` and `run.sh` are executable.
- [x] GitHub repository created: `https://github.com/3d3dcanada/ora-rust`

## Current Status
- The crate builds in debug and release modes. `cargo build --release` completed successfully and produced `target/release/ora`.
- The automated test suite is not release-clean. `cargo test` fails on `security::gates::tests::test_latency_is_under_2ms` with an observed latency of `3021 us`.
- `cargo fmt --check` fails across a large portion of the tree, so formatting is not normalized for release.
- API mode starts and responds on port `8001`, and MCP stdio mode answers `initialize` and `tools/list`.
- The repository at `/home/wess/ai-workspace/ora-rust` is not a Git repository right now, so nothing here can be committed or pushed to GitHub until Git metadata is restored or initialized.

## Verified Blockers

### 1. Git/GitHub Blocker
- There is no `.git` directory in `/home/wess/ai-workspace/ora-rust`.
- Immediate consequence: no branch state, no diff history, no remote, no push.
- First release task is to reconnect this directory to the intended Git repository or initialize a new repository and add the correct GitHub remote.

### 2. Release Gating Failures
- `cargo test` is red because the AST parser latency benchmark currently misses the stated `< 2ms` target.
- `cargo fmt --check` is red.
- The codebase also emits a large number of compiler warnings that should be reduced before release.

### 3. Runtime/Product Gaps
- `/chat` and `/kernel/process` currently depend on `Kernel::process_with_llm()`, which hardcodes Ollama model `llama3.2` at `http://localhost:11434`. In this environment Ollama is running, but that model is missing, so `/chat` returns an LLM error instead of a valid response.
- `/router/models` returns a static hardcoded model list instead of discovered provider/model state.
- `/security/status`, `/authority/current`, `/authority/escalate`, `/approvals`, and `/kernel/metrics` currently return placeholder/static values rather than authoritative live state.
- `web_search` in the tool executor is explicitly a placeholder.
- WebSocket cancellation is not implemented.

### 4. Documentation/Packaging Drift
- `TODO.md` marks several major phases as complete even though important runtime and validation gaps remain.
- `build.sh` and `run.sh` contain valid shebang scripts but are not executable on disk.
- The binary has no explicit help/CLI contract; `cargo run -- --help` starts the server instead of returning usage output.

## File-Level Evidence
- Version is still `0.1.0`: `Cargo.toml`
- Test checklist still incomplete: `TODO.md`
- CLI startup only distinguishes `--mcp-mode` versus default HTTP mode: `src/main.rs`
- Latency assertion that currently fails: `src/security/gates.rs`
- Hardcoded Ollama path/model in request path: `src/kernel/mod.rs`
- Static API responses: `src/gateway/http.rs`
- Placeholder tool implementation: `src/kernel/tools.rs`
- WebSocket cancellation TODO: `src/gateway/websocket.rs`

## Completion Plan

### Phase 1: Restore Pushability
1. Restore or initialize Git for `/home/wess/ai-workspace/ora-rust`.
2. Confirm the intended GitHub remote and default branch.
3. Add or verify `.gitignore` coverage for `target/`, local vaults, audit logs, temp outputs, and config secrets.
4. Capture the current working tree as the baseline before any release edits.

### Phase 2: Make the Build Release-Clean
1. Fix the AST parser latency regression or revise the benchmark so it measures a stable, defensible SLA.
2. Run `cargo fmt` and commit to a single formatting baseline.
3. Reduce compiler warnings, prioritizing unused imports/variables and ignored `Result`s.
4. Re-run `cargo test` until green.

### Phase 3: Fix Functional Release Blockers
1. Replace the hardcoded `Kernel::process_with_llm()` model/provider path with configuration-driven provider selection.
2. Make `/chat` succeed with the configured local model set or surface a clear startup/configuration error before requests are accepted.
3. Replace static `/router/models` output with actual discovered provider/model availability.
4. Replace placeholder/static status endpoints with real state reads, or cut those endpoints from the release surface if they are not ready.
5. Decide whether `web_search` ships as a real integration or is removed from the advertised tool list.

### Phase 4: Finalize MCP and Realtime Behavior
1. Expand MCP tools/resources only to the subset that is actually implemented and supportable.
2. Add tests for graph transitions and MCP tool-call flows.
3. Implement or explicitly disable WebSocket cancellation and approval flows.
4. Run a manual Claude Desktop or compatible MCP client smoke test against `target/release/ora --mcp-mode`.

### Phase 5: Align Docs, Scripts, and Versioning
1. Update `README.md`, `TODO.md`, and the existing planning docs to reflect actual implementation status.
2. Mark `build.sh` and `run.sh` executable, or replace them with documented `cargo` commands.
3. Add a real CLI help path and explicit startup modes.
4. Bump the crate version only after test/build/docs are aligned.

### Phase 6: Final Validation Before GitHub Push
1. `cargo fmt --check`
2. `cargo test`
3. `cargo build --release`
4. API smoke test: `/health`, `/chat`, `/router/models`, `/security/status`
5. MCP smoke test: `initialize`, `tools/list`, at least one `tools/call`
6. Review secrets/config handling to ensure nothing local is committed
7. Commit, tag if needed, and push to GitHub once the directory is under Git again

## Recommended Immediate Order
1. Reattach or initialize Git in this directory.
2. Fix the failing latency test and formatting drift.
3. Replace the hardcoded `llama3.2` execution path so `/chat` works against the models actually installed.
4. Remove or replace placeholder/static API responses.
5. Run the full validation sequence and only then prepare the GitHub push.

## Release Exit Criteria
- Directory is a valid Git repository with the correct GitHub remote.
- `cargo fmt --check`, `cargo test`, and `cargo build --release` all pass.
- `/chat` returns a valid model response in the default environment.
- MCP mode works with at least one real tool call.
- Docs match the product that actually ships.
- Version is bumped intentionally from `0.1.0` after validation, not before.
