# ORA-Rust: Official Benchmarking Plan

## 1. Why Benchmarks Matter for ORA
In late 2026, AI claims are mostly noise. "10x faster" means nothing without reproducible, open-source benchmarks. ORA's primary engineering pivot from Python to Rust must be proven statistically. We must publish our benchmarks directly on our website and in our GitHub README.

## 2. The 3 Core Benchmark Metrics

### A. Orchestration Latency (The "Rust Premium")
Python-based agents (CrewAI, AutoGen, LangGraph) suffer from significant GIL (Global Interpreter Lock) constraints and memory overhead during multi-agent routing.
*   **The Test:** A 5-agent routing task (Planner -> Search -> Summarize -> Quality Check -> Output) simulated with a mock LLM (0ms API response time) to test *pure framework overhead*.
*   **Competitors to Test Against:** `CrewAI` (Python), `AutoGen` (Python), `LangGraph` (Python).
*   **Target ORA-Rust Result:** < 2ms framework overhead (compared to 50-100ms in Python).

### B. Scalable Context Retrieval (The "Zep / Mem0 Challenger")
We must prove that our embedded vector memory (Pulz-Rust) scales gracefully.
*   **The Test:** Querying a Top-K similarity search against a 1 Million, 10 Million, and 50 Million token corpus.
*   **Competitors to Test Against:** `Mem0` (Self-hosted), `Motorhead` (Rust server), `Zep`.
*   **Target ORA-Rust Result:** P99 latency under 15ms at 1M tokens.

### C. Prompt Injection Block Rate (The "A0-A5 Shield")
This is ORA's unique selling proposition. We cannot just claim we are secure; we must prove it against standardized attack vectors.
*   **The Test:** Run standard open-source adversarial test suites (e.g., `Garak`, `PyRIT`) containing 5,000 known Prompt Injections, Jailbreaks, and IDPI (Indirect Prompt Injection) payloads through the Ora Security Gates.
*   **Competitors to Test Against:** A raw OpenAI API call, a raw Claude API call, and a `Sema4.ai` deployment (if testable).
*   **Target ORA-Rust Result:** 99.9% block rate of AST-layered malicious payloads, with a false positive rate (blocking legitimate queries) of < 1%.

## 3. "The Strawberry Protocol" Logic Benchmark
To demonstrate ORA's "Cross-Model Verification Loop" (as discussed in the marketing plan regarding the "R in strawberry" hallucination problem):
*   **The Test:** A custom benchmark of 50 common LLM logic-trap questions (e.g., character counts, exact math, temporal ordering).
*   **The Run:** Pass the benchmark through standard DeepSeek V4 / Claude 3.5 without ORA, then pass them *through* ORA with the verification loop engaged.
*   **Target Result:** Demonstrate a 40%+ increase in logic puzzle accuracy simply by routing the raw LLM through ORA's verification gates.

## 4. Publishing Strategy
1.  Open-source the benchmark orchestrator code (e.g., `ora-benchmarks` repo) so the community can run it themselves.
2.  Use the `ora_benchmark_abstract` generated image as the hero image for a dedicated `/benchmarks` page on the website.
3.  Publish a detailed engineering blog post: "How we achieved 2ms agent orchestration latency in Rust."
