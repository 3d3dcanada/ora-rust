# ORA-Rust: Codebase Finalization TODO

This document serves as the handoff and execution guide for the next agent or developer finalizing the `ora-rust` codebase.

Our goal is to pivot from the marketing/planning phase into hardcore Rust engineering. We must make `ora-rust` a portable, highly secure AI orchestrator capable of running as an API or a local MCP server.

## Execution Checklist

### Phase 1: The Core Security Kernel (Type-State)
- [x] Investigate `src/kernel/constitution.rs` and `src/kernel/authority.rs`.
- [x] Implement the `A0` through `A5` structs. Define generic state bounds on secure operations (e.g., `impl SecureAction<A5> { ... }`).
- [x] Write `trybuild` or standard unit tests to verify that invalid permission contexts fail at compile time.
- [x] Finalize `src/security/vault.rs` using a secure cryptography crate (`ring` or `rustls`) to handle secret storage.

### Phase 2: Security Gates & Validation
- [x] Implement `src/security/gates.rs` -> The AST Parser. It must intercept incoming prompts and evaluate them for injection patterns.
- [x] Implement `src/security/sandbox.rs` -> The IDPI (Indirect Prompt Injection) defense. Sanitize all incoming data from web requests or file reads before joining it to the LLM context string.
- [x] Benchmark the AST Parser to ensure it hits the marketed `< 2ms` latency. 

### Phase 3: The Multi-Agent DAG Router
- [x] Create `src/orchestration/mod.rs`, `graph.rs`, and `agent.rs`.
- [x] Implement a Directed Acyclic Graph (DAG) for agent routing using `petgraph`.
- [x] Define the `Agent` trait representing standard nodes (e.g., Planner, Researcher, Coder).
- [x] Ensure the router processes context through the security gates between every node hop.

### Phase 4: Provider-Agnostic LLM Layer
- [x] Expand `src/llm/client.rs` into a unified `LlmProvider` async trait.
- [x] Implement `src/llm/providers/deepseek.rs` (DeepSeek V4).
- [x] Implement `src/llm/providers/anthropic.rs` (Claude 3.5).
- [x] Implement `src/llm/providers/local.rs` (OpenAI compatible generic client for Ollama/vLLM).

### Phase 5: The MCP / API Gateway
- [x] Build `src/gateway/mcp.rs`. This must consume stdin/stdout (or SSE) to serve as a Model Context Protocol server.
- [x] Expose ORA's internal secure tools via the standard MCP JSON-RPC format.
- [x] Ensure `src/main.rs` can start the application in either `--mcp-mode` (stdio) or `--api-mode` (Axum web server).

### Phase 6: Testing & Validation
- [ ] Write unit tests for all graph transitions.
- [ ] Perform a manual local integration test using Claude Desktop connected to the newly built `ora-rust` binary.
- [ ] Validate memory usage and latency compared to historical Python benchmarks.
