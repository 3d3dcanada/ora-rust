# Ora-Rust Finalization & Architecture Plan (March 2026)

## 1. Executive Summary
This document outlines the finalization architecture required to bring `ora-rust` from its current backend scaffolding to a fully operational, market-ready Enterprise AI Operating System and secure MCP Server. By combining the safety of Rust with the robust multi-agent orchestration of the original Python `ora` codebase, `ora-rust` will be positioned as the premier "Constitutional AI Server" heading into late 2026.

## 2. Core Ora Porting Roadmap
The original Python `ora` codebase relied heavily on a complex orchestration layer. To ensure `ora-rust` is "solid as f***," the following core functionalities must be meticulously ported and hardened in Rust:

### A. The Constitutional Security Kernel (A0-A5)
*   **Status in `ora-rust`:** Stubbed in `src/kernel/constitution.rs` and `src/kernel/authority.rs`.
*   **Finalization Needed:**
    *   **Strict Type-State Pattern:** Implement the A0-A5 clearance levels using Rust's type system to ensure compile-time security. A function requiring A4 clearance should physically not compile if passed an A2 context.
    *   **Cryptographic Vault (`src/security/vault.rs`):** Must be finalized to use post-quantum cryptography (e.g., `ring` or `rustls` with Kyber support) for storing API keys and agent identity materials, preventing exfiltration even in the event of a total server breach.

### B. Security Gateways & Malicious Prompt Injection Prevention
*   **Status in `ora-rust`:** Base structure in `src/security/gates.rs`.
*   **Finalization Needed (March 2026 Standards):**
    *   **Multi-Layered Defense:** As established by recent 2026 cybersecurity consensus (NCSC norms, Meta SecAlign-70B research), singular boundary filters fail. Ora-Rust must implement layered AST (Abstract Syntax Tree) parsing of prompts.
    *   **Instruction Obfuscation Detection:** Implement rust-based heuristic scanners to catch homoglyphs, fragmentation, and encoded payload attacks before they reach the LLM.
    *   **IDPI (Indirect Prompt Injection) Sandbox:** Web-fetched content (via tools) must be handled in a separate, isolated memory allocation context. Tool outputs must be stripped of executable intent before being re-injected into the context window. 
    *   **Grounding verification:** Post-generation logic checks to ensure outputs conform to established truth (mitigating the "R in strawberry" tokenization failure by forcing cross-model verification loops for logic tasks).

### C. Multi-Agent Graph Routing
*   **Status in `ora-rust`:** Missing.
*   **Finalization Needed:**
    *   Port the graph-based routing from Python's `orchestrator` and `router`.
    *   **Rust Implementation:** Utilize `petgraph` or a custom DAG (Directed Acyclic Graph) engine in Rust to manage agent states (Planner -> Researcher -> Builder). This will be exponentially faster and more memory-safe than the Python equivalent.

### D. Provider-Agnostic LLM Client Layer
*   **Status in `ora-rust`:** Basic framing in `src/llm/client.rs`.
*   **Finalization Needed:**
    *   Build a unified trait (`LlmProvider`) capable of hot-swapping between:
        *   **DeepSeek V4:** (Crucial for March 2026: V4's domestic hardware optimization and 40% memory reduction architecture). We must support DeepSeek natively for the Asian market and cost-conscious enterprise.
        *   **Moonshot Kimi K2.5:** Support its 256K context window and native "agent swarm" APIs.
        *   **Anthropic Claude 3.5 / OpenAI GPT-4.5/5.**
        *   **Local Models (Ollama/vLLM):** For distinct, off-grid deployments.

## 3. The Enterprise Additions (MCP & Memory)

### A. Native MCP Server Integration
*   To compete with managed servers like CData Connect or Amazon Bedrock AgentCore, `ora-rust` must natively expose an Anthropic Model Context Protocol (MCP) compatible endpoint (`/mcp/v1/...`).
*   **Implementation:** Build a middleware layer in Axum (`src/gateway/mcp.rs`) that translates external MCP requests (like file system access or memory retrieval) into Ora's internal, A0-A5 governed tool calls.

### B. High-Speed Vector Memory (Pulz Equivalent)
*   **Implementation:** Integrate a fast, embedded Rust vector database (e.g., `qdrant-rust` or a lighter embedded HNSW library) to handle the `pulz` memory layer, facilitating sub-millisecond context retrieval for RAG operations.

## 4. Operational & Deployment Architecture
*   **Lightweight Target:** The resulting binary must be minimal. Since Vercel hosting is preferred for the frontend and API gateway, the Rust backend should be compiled to a static binary (`target/release/ora-rust`) or a tiny WebAssembly (Wasm) module if deployed on edge networks, though a sustained Docker deployment on AWS/DigitalOcean/Fly.io is recommended for the persistent memory/database connections.
*   **Telemetry:** Integrate OpenTelemetry in Rust for enterprise observability (tracing tool calls, token usage, and latency).

## 5. Summary Check-Off Before Market Launch
1.  [ ] Rust Type-Safe A0-A5 Implementations verified.
2.  [ ] Multi-Layer Prompt Injection Scanner (Rust-native) complete.
3.  [ ] Multi-Agent DAG router ported and tested.
4.  [ ] DeepSeek V4 & Kimi K2.5 API adapters written.
5.  [ ] MCP Server Endpoint (`/mcp`) active and responding to Claude.
