# ORA-Rust Master Plan 2026

## 1. Purpose

This document is the strategic source of truth for turning `ora-rust` into the real ORA platform rather than a thinner Rust rewrite of selected security ideas. It supersedes lighter planning notes by defining:

- what ORA actually is,
- what must be preserved from the original Python system,
- what should be borrowed from adjacent PulZ work,
- what must be cut,
- how the Rust implementation should be staged,
- and how success will be measured.

The core premise is simple:

`ORA must feel like one highly capable assistant while internally behaving like a disciplined, token-efficient, specialist reasoning system.`

That means the user should not be forced to micromanage browsing, verification, routing, or specialist selection. ORA should determine the cheapest trustworthy path to an answer or action, using memory, local context, cached evidence, live retrieval, and internal sub-runs only when they are needed.

## 2. What ORA Is

ORA is not primarily:

- a chatbot wrapper,
- a browser-first agent,
- a constitution-only security gate,
- a generic MCP server,
- or a visible swarm of agents the user must orchestrate manually.

ORA is:

- an operational reasoning layer,
- a routing and decomposition engine,
- a memory and compression system,
- a retrieval and acquisition system,
- a silent verification system,
- and an interface unifier that presents all of this as one coherent assistant.

The original Python ORA points in this direction through its smart router, Pulz memory layer, agent specialization concepts, and cost-aware multi-model design. The audit of the Python system showed the primary failure was not product thesis; it was incomplete integration. Tools, memory, routing, and agent concepts existed, but they were not fully wired into a trustworthy operational path.

The Rust system should correct that by making the internal path explicit, typed, measurable, and production-grade.

## 3. Product Thesis

The strongest market position for ORA in 2026 is:

`one assistant, internally optimized like a swarm, externally simple like a professional tool`

That product thesis has six pillars.

### 3.1 Single Face, Multi-Path Core

The user interacts with one ORA service through MCP, API, CLI, or future UI surfaces. Internally, ORA may perform classification, retrieval, memory recall, verification, planning, tool execution, and specialist sub-runs. These internal stages should remain largely invisible unless surfaced for audit or operator inspection.

### 3.2 Cheapest Trustworthy Path

The system should not default to the shortest plausible answer path. It should optimize for:

- low token cost,
- low latency,
- sufficient freshness,
- sufficient verification,
- and acceptable risk for the task class.

This means ORA may choose:

- memory only,
- local repository/document retrieval,
- cached web evidence,
- a partial live browse,
- or a deeper multi-step internal run.

### 3.3 Silent Verification

Verification should be default behavior, not a user burden. The user should not have to repeatedly ask for browsing, sources, or cross-checking. ORA should infer when verification is required based on:

- task type,
- recency sensitivity,
- claim density,
- risk profile,
- missing support,
- and contradiction signals.

### 3.4 Token Intelligence

ORA should outperform generic assistants in the amount of work completed per token consumed. It should aggressively reuse:

- semantic memory,
- distilled evidence,
- prior route outcomes,
- compressed task state,
- and compact intermediate summaries.

### 3.5 Security Without Constant Friction

The constitution and approval system must be real, but it cannot make normal usage feel bureaucratic. Most low-risk work should remain smooth. Higher-risk actions should trigger stronger controls only when the risk class demands it.

### 3.6 Operator Visibility

The system should expose route choice, evidence origin, confidence movement, and mission state for operators, while preserving a clean default experience for end users.

## 4. Strategic Interpretation of Existing Assets

### 4.1 Original Python ORA

Keep:

- smart routing as a first-class concept,
- Pulz-style memory and recall,
- specialist decomposition,
- provider-agnostic operation,
- internal cost optimization,
- the ambition for one outward assistant with multiple internal competencies.

Do not copy blindly:

- stub-heavy orchestrator flows,
- tool systems that are defined but not actually executed,
- heuristic verification marketed as stronger than it is,
- oversized architecture claims without runtime proof.

### 4.2 Current `ora-rust`

Keep:

- Rust as the implementation base,
- the emerging constitution, authority, security, orchestration, gateway, and tool namespaces,
- MCP as a delivery surface,
- the desire for typed safety and low overhead.

Correct immediately:

- direct chat paths that bypass the intended orchestration,
- placeholder retrieval and web tooling,
- shallow DAG execution,
- unimplemented MCP resource models,
- weak integration between routing, tools, verification, and memory.

### 4.3 PulZ, PulZ-Buzz, and the Dashboard

Useful ideas to absorb:

- typed kernel primitives such as evidence items, evidence reports, decision frames, and append-only logs,
- mission-oriented acquisition patterns,
- operator-facing dashboards and feeds,
- separation between core kernel logic and operator surfaces.

Do not inherit as-is:

- marketing-heavy trust formulas presented as hard verification,
- heuristic graph or trust modules treated as sufficient for enterprise truth guarantees,
- architecture that privileges governance language over actual routing and retrieval performance.

Net takeaway:

PulZ provides vocabulary and operator patterns.
ORA provides the actual product thesis.
Rust must supply the integration discipline.

## 5. Target System Shape

The production system should be organized around six kernels. These are internal kernels, not necessarily top-level crates on day one, but they should be explicit architectural boundaries.

### 5.1 Route Kernel

Responsibilities:

- classify task intent,
- detect freshness requirements,
- detect required capability classes,
- select memory vs retrieval vs browse vs specialist decomposition,
- choose the execution depth,
- track route cost and route quality.

The route kernel is the true ORA core. If this is wrong, the rest of the system becomes expensive theater.

### 5.2 Memory Kernel

Responsibilities:

- maintain durable semantic memory,
- store compressed working memories,
- retain distilled task artifacts,
- remember prior route outcomes,
- expose evidence-aware recall rather than only conversational recall.

Memory should reduce repeated token spend and repeated browsing. It should also support continuity across projects, repos, domains, and missions.

### 5.3 Acquisition Kernel

Responsibilities:

- run targeted web and document retrieval,
- support mission-based crawling,
- normalize source artifacts,
- maintain provenance and timestamps,
- feed the evidence graph and caches.

This is where lucrative web crawling belongs. Not as uncontrolled scraping, but as governed acquisition and refresh missions across:

- public documentation,
- package registries,
- issue trackers,
- research papers,
- regulatory sources,
- court or policy material where permitted,
- domain-specific feeds,
- and internal approved corpora.

### 5.4 Verification Kernel

Responsibilities:

- verify claims against evidence,
- detect stale or conflicting evidence,
- label unsupported claims,
- downgrade or refuse when required,
- validate tool outcomes where possible,
- produce a compact evidence bundle for the compiler.

This kernel should be quiet during smooth operation and loud only when support is weak or risk is high.

### 5.5 Constitution Kernel

Responsibilities:

- classify risk and authority,
- gate tools and resources,
- enforce approval thresholds,
- constrain execution classes,
- and preserve auditability.

This kernel must be deterministic. The model may propose; the constitution authorizes or blocks.

### 5.6 Operator Kernel

Responsibilities:

- show route decisions,
- inspect evidence bundles,
- track mission state,
- display confidence changes,
- expose failures, contradictions, and approval bottlenecks.

The operator kernel is how ORA becomes enterprise-grade without pushing complexity directly into every end-user workflow.

## 6. Execution Model

Every meaningful request should travel through a staged execution model, though some stages may be nearly no-op depending on the route.

### 6.1 Proposed Standard Path

1. Intake and task classification.
2. Task profile derivation:
   - freshness sensitivity,
   - authority class,
   - required capability,
   - expected evidence burden,
   - likely output type.
3. Memory sufficiency check.
4. Local corpus sufficiency check.
5. Cached evidence sufficiency check.
6. Live acquisition if required.
7. Specialist decomposition if required.
8. Draft synthesis.
9. Verification pass.
10. Constitution and approval checks for tool actions.
11. Final compilation.
12. Audit and memory distillation.

### 6.2 Route Classes

To avoid over-engineering every request, ORA should support route classes.

- `R0: Memory answer`
  Use durable memory or compressed task state only.
- `R1: Local grounded answer`
  Use repository files, saved docs, or approved local sources.
- `R2: Cached evidence answer`
  Use local evidence bundles or recent crawled results.
- `R3: Live retrieval answer`
  Use current web or external retrieval.
- `R4: Specialist run`
  Use planner plus one or more specialists.
- `R5: Action run`
  Execute tools or workflows under constitutional control.

The route kernel should escalate only when confidence, freshness, or capability requirements demand it.

## 7. Verification Philosophy

ORA should not promise the impossible. It cannot eliminate hallucination at the model layer. It can, however, sharply reduce unsupported outputs by refusing to finalize claims that lack sufficient support.

### 7.1 Practical Verification Contract

For factual answers:

- every material claim should map to evidence or be labeled unsupported,
- stale evidence should be flagged,
- conflicting evidence should be surfaced rather than hidden,
- time-sensitive claims should prefer live or recent evidence,
- high-risk answers should have stricter thresholds.

For coding:

- repository claims should be backed by file references,
- proposed changes should be validated against local code structure,
- tool outputs should be checked before being summarized as success.

For legal, medical, and research:

- ORA should default to source-linked, evidence-bounded answers,
- it should avoid overstating certainty,
- and it should surface known uncertainty rather than smooth it away.

### 7.2 Verification Modes

Support explicit system modes.

- `Light`
  Silent checks, low friction, suitable for standard coding or research support.
- `Standard`
  Evidence-backed final answers for most serious workflows.
- `Strict`
  Strong refusal or uncertainty labeling when support is incomplete.
- `Regulated`
  Strongest sourcing, freshness, and audit requirements.

## 8. Token Efficiency Strategy

Token efficiency is not a side optimization. It is a core differentiator.

### 8.1 Required Tactics

- semantic caching of route outcomes,
- evidence bundle reuse,
- prompt-frame compression,
- chunk-level retrieval instead of raw document stuffing,
- summary distillation after each internal stage,
- partial browsing instead of indiscriminate browsing,
- compact model instructions tuned by route class,
- optional small local models for classification or filtering.

### 8.2 Anti-Patterns to Avoid

- browsing first for every nontrivial question,
- dumping full documents into the final context,
- re-running identical sub-questions,
- storing verbose memory that is never distilled,
- forcing all tasks through full multi-agent choreography.

## 9. Acquisition and Crawling Strategy

The acquisition kernel is where ORA can become materially stronger than generic MCP tooling.

### 9.1 Why ORA Needs Crawling

Search alone is reactive. Crawling creates owned knowledge flows:

- domain watchlists,
- update monitoring,
- evidence refresh jobs,
- competitor tracking,
- package and changelog surveillance,
- legal and research source capture,
- and niche knowledge accumulation.

### 9.2 Required Properties

- allowlisted sources,
- explicit provenance,
- robots and policy compliance,
- recrawl scheduling,
- deduplication,
- content hashing,
- snapshot retention,
- contradiction detection,
- and structured extraction into an evidence graph.

### 9.3 Output Model

Crawlers should not just dump HTML. They should produce:

- normalized source artifacts,
- extracted entities,
- extracted claims,
- timestamps,
- trust metadata,
- and linkable evidence items.

### 9.4 Enterprise Value

This is how ORA becomes a researcher’s fast path, a lawyer’s evidence assistant, a coder’s continuously refreshed dependency intelligence layer, and an operator’s competitive monitoring system.

## 10. Provider Resilience

The system should be resilient to provider shifts, changing policies, rate limits, and degraded capabilities.

The correct design goal is not bypass. It is independence.

### 10.1 Requirements

- provider-agnostic model interfaces,
- local or open-weight fallbacks,
- retrieval and memory that remain useful even when a provider changes,
- route policies that can downgrade gracefully,
- exportable memory and evidence stores,
- minimal dependence on one hosted vendor.

### 10.2 Why This Matters

If providers get more restrictive, slower, or weaker in certain tasks, ORA should still operate because the real asset is not a single API call. The asset is the route engine, memory, evidence, and internal execution model.

## 11. Security and Governance

Security remains a core pillar, but it must support ORA’s real operating model rather than overshadow it.

### 11.1 What Must Be Strong

- tool gating,
- approval routing,
- prompt injection resistance,
- source sanitization,
- workspace boundaries,
- secret isolation,
- audit trails,
- deterministic policy checks.

### 11.2 What Must Not Happen

- governance becoming the dominant product story,
- security layers forcing needless user friction on low-risk tasks,
- “trust scores” replacing actual evidence handling,
- policies living only in prompts instead of deterministic code paths.

## 12. Benchmarks and Success Metrics

ORA should be judged on measurable system behavior, not architecture language.

### 12.1 Core Metrics

- route accuracy,
- verification hit rate,
- unsupported claim rate,
- evidence reuse rate,
- average token cost per task class,
- p50 and p95 latency by route class,
- approval interruption rate,
- crawl freshness,
- contradiction detection rate,
- and operator visibility completeness.

### 12.2 Critical Benchmarks

For coding:

- can ORA answer repo questions with fewer tokens than a baseline assistant,
- can it correctly choose file-grounded answers over generic chat answers,
- can it verify tool success reliably.

For research:

- can ORA detect when browsing is required,
- can it distinguish stale from fresh evidence,
- can it summarize conflict rather than average it away.

For regulated or high-stakes use:

- does it refuse unsupported answers,
- can it produce a verifiable evidence bundle,
- does it preserve an audit path.

## 13. Non-Goals

To keep the product honest, the following are non-goals for the first serious ORA-Rust build.

- claiming formal truth guarantees for broad natural language answers,
- building an always-visible consumer swarm UI,
- chasing every frontier model feature before the route engine works,
- replacing domain professionals,
- or turning ORA into a generic no-boundaries autonomous bot.

## 14. Delivery Phases

### Phase 0: Reality Alignment

Objectives:

- define ORA correctly,
- identify dead abstractions,
- lock the kernel map,
- and stop building around generic “constitutional AI” branding alone.

Exit criteria:

- the team aligns on the six-kernel model,
- route-first architecture is accepted,
- placeholder trust language is downgraded to concrete operational requirements.

### Phase 1: Route Kernel Foundation

Objectives:

- replace direct-answer shortcuts,
- introduce route classes,
- add sufficiency checks for memory, local corpus, cached evidence, and live retrieval,
- record route decisions and reasons.

Exit criteria:

- every request follows a declared route class,
- route decisions can be inspected,
- simple tasks avoid unnecessary browsing or heavy orchestration.

### Phase 2: Memory Kernel

Objectives:

- add durable semantic memory,
- add compressed task memory,
- add evidence-aware recall,
- build memory distillation after each completed task.

Exit criteria:

- repeated tasks show lower token spend,
- prior work can be recalled with provenance,
- memory quality is inspectable.

### Phase 3: Acquisition Kernel

Objectives:

- implement real web retrieval,
- add saved-doc and local corpus retrieval,
- build mission-based crawling,
- normalize sources into evidence items.

Exit criteria:

- placeholder web search is gone,
- crawl artifacts are reusable,
- freshness metadata exists for retrieved evidence.

### Phase 4: Verification Kernel

Objectives:

- implement evidence-backed claim verification,
- support unsupported-claim labeling,
- surface contradiction and stale evidence,
- support stricter profiles for regulated work.

Exit criteria:

- high-risk routes cannot finalize unsupported answers silently,
- evidence bundles can be attached to results,
- contradiction handling works.

### Phase 5: Constitution and Action Control

Objectives:

- strengthen deterministic policy enforcement,
- connect action classes to route and evidence state,
- integrate approval pathways cleanly,
- tie action gating to real execution contexts.

Exit criteria:

- risky tools are never executed outside policy,
- low-risk work stays smooth,
- audit log captures the decision path.

### Phase 6: Operator Kernel and Product Surface

Objectives:

- expose route traces,
- expose crawl missions and artifacts,
- expose verification status,
- create an operator-grade dashboard or feed model.

Exit criteria:

- operators can inspect why ORA chose a path,
- evidence and contradictions are reviewable,
- enterprise workflows can monitor system behavior.

### Phase 7: Market Readiness

Objectives:

- benchmark against baseline assistants and MCP stacks,
- tighten latency and token efficiency,
- define packaging and deployment,
- lock the product narrative around one assistant with internal intelligence.

Exit criteria:

- benchmark wins or justified tradeoffs are documented,
- the story matches the actual runtime behavior,
- the system is ready for external pilots.

## 15. Immediate Direction for This Repository

For `ora-rust` specifically, the highest-priority correction is:

`stop treating the current architecture as nearly done`

The current repo has useful scaffolding, but it is still missing the deep operational path that makes ORA real.

Immediate focus should be:

- route kernel before more branding work,
- real retrieval before more trust language,
- memory before more agent names,
- verification tied to evidence before more heuristic scoring,
- operator visibility after the inner path is functioning.

## 16. Approval Checkpoint

If this master plan is approved, the next implementation artifacts should be:

1. a source-of-truth execution backlog,
2. a kernel-by-kernel file map for `ora-rust`,
3. a milestone 1 implementation plan centered on the route kernel,
4. and a cleanup pass that removes misleading “done” items from the current execution narrative.

