# ORA-Rust Execution Backlog 2026

## 1. How To Use This Backlog

This backlog translates the master plan into execution work. It is intentionally detailed so it can serve as:

- an approval artifact,
- a milestone tracker,
- a session handoff reference,
- and a reality check against vague architectural claims.

Rules for use:

- nothing is marked complete until it is implemented and verified in this repository,
- placeholder code does not count as delivery,
- benchmark claims do not count unless measured,
- and marketing language must never outrun runtime reality.

## 2. Priority Model

- `P0`: foundational, blocks product truth.
- `P1`: critical, needed for first serious ORA build.
- `P2`: important, but only after the core path works.
- `P3`: valuable polish or expansion.

## 3. Milestone 0: Repo Reality Reset

### M0.1 Correct the narrative

- `P0` Document that current `ora-rust` is scaffolding, not a near-finished enterprise kernel.
- `P0` Identify every placeholder or bypass path in the current request flow.
- `P0` Identify every architectural component that exists only nominally.
- `P1` Mark legacy docs that are advisory rather than authoritative.

Acceptance criteria:

- there is one approved source-of-truth plan,
- direct model shortcuts are documented,
- “done” language in docs no longer overstates runtime reality.

### M0.2 Freeze the kernel map

- `P0` Define the six kernel boundaries: route, memory, acquisition, verification, constitution, operator.
- `P0` Map existing files into `keep`, `rewrite`, `replace`, `defer`, or `remove`.
- `P1` Note where PulZ primitives can be adapted cleanly.

Acceptance criteria:

- each major module has a declared role,
- duplicate or misleading concepts are identified,
- the team can explain ORA in one consistent way.

## 4. Milestone 1: Route Kernel

### M1.1 Replace direct answer shortcuts

- `P0` Remove or isolate any path where requests bypass orchestration and go straight to a default chat client.
- `P0` Introduce a route decision object with:
  - task class,
  - freshness requirement,
  - risk class,
  - expected evidence burden,
  - selected route class,
  - and route reason.
- `P0` Ensure every request receives a route decision before synthesis.

Acceptance criteria:

- there is no silent default-answer path,
- route decisions are recorded,
- route choice is inspectable in logs or traces.

### M1.2 Add route classes

- `P0` Implement declared route classes `R0` through `R5`.
- `P1` Define escalation rules between route classes.
- `P1` Ensure low-risk simple tasks do not trigger unnecessary heavy paths.

Acceptance criteria:

- repeated examples consistently choose the same route class when conditions match,
- route escalation is deterministic and explainable.

### M1.3 Add sufficiency checks

- `P0` Implement memory sufficiency checks.
- `P0` Implement local corpus sufficiency checks.
- `P0` Implement cached evidence sufficiency checks.
- `P1` Implement live retrieval necessity checks.

Acceptance criteria:

- ORA can justify why it did or did not browse,
- ORA prefers cheaper trustworthy sources before escalating.

## 5. Milestone 2: Memory Kernel

### M2.1 Durable semantic memory

- `P0` Choose the storage model for durable memory.
- `P0` Define memory record types:
  - conversation distillations,
  - project facts,
  - evidence summaries,
  - route outcomes,
  - operator notes.
- `P1` Add memory retrieval by semantic relevance plus freshness and provenance.

Acceptance criteria:

- memory recall returns compact, relevant, attributable results,
- memory can reduce repeated context injection.

### M2.2 Working memory compression

- `P0` Add a compact task-state structure for ongoing work.
- `P1` Distill internal chain state into reusable summaries after each task.
- `P1` Avoid storing raw verbose interaction history as the primary recall path.

Acceptance criteria:

- repeated follow-up tasks consume fewer tokens than cold starts,
- working memory remains small and legible.

### M2.3 Memory quality controls

- `P1` Track memory staleness.
- `P1` Track memory confidence and origin.
- `P2` Add memory repair or overwrite rules when evidence changes materially.

Acceptance criteria:

- stale memory is not treated as fresh fact,
- updated evidence can supersede old distilled memory.

## 6. Milestone 3: Acquisition Kernel

### M3.1 Real retrieval connectors

- `P0` Replace placeholder web search with actual retrieval.
- `P0` Add first-class local document retrieval.
- `P0` Add first-class repository retrieval.
- `P1` Add connector abstraction for future domain-specific retrieval sources.

Acceptance criteria:

- retrieval results carry source, timestamp, and content metadata,
- local docs and repository context are treated as first-class evidence.

### M3.2 Evidence normalization

- `P0` Define evidence item primitives for all acquisition outputs.
- `P0` Add hashing, timestamps, source type, and sanitizer state.
- `P1` Add extraction of entities, claims, and reference snippets.

Acceptance criteria:

- all retrieval outputs can be normalized into a common evidence model,
- downstream verification does not need source-specific code paths for basic handling.

### M3.3 Mission-based crawling

- `P1` Add crawl mission definitions:
  - scope,
  - frequency,
  - freshness policy,
  - extraction rules,
  - and storage policy.
- `P1` Add mission artifact storage.
- `P2` Add scheduler and recrawl logic.
- `P2` Add contradiction and update detection across crawl generations.

Acceptance criteria:

- crawl missions can be run repeatedly,
- outputs become reusable evidence bundles,
- changes across time are detectable.

## 7. Milestone 4: Verification Kernel

### M4.1 Claim-aware verification

- `P0` Define what constitutes a claim in ORA outputs.
- `P0` Add support mapping from claims to evidence items.
- `P0` Add unsupported-claim labeling or suppression.
- `P1` Add contradiction surfacing.

Acceptance criteria:

- substantive factual outputs cannot finalize without either support or explicit uncertainty,
- contradictory evidence is surfaced rather than smoothed away.

### M4.2 Freshness-aware verification

- `P0` Add freshness windows by task type.
- `P1` Require live or recent evidence for time-sensitive domains.
- `P1` Downgrade confidence when only stale support exists.

Acceptance criteria:

- ORA distinguishes historical fact from current fact,
- stale support triggers the correct downgrade path.

### M4.3 Domain profiles

- `P1` Add verification profiles for:
  - general,
  - coding,
  - research,
  - legal support,
  - healthcare support.
- `P2` Tune evidence burden and refusal behavior by profile.

Acceptance criteria:

- stricter profiles materially change system behavior,
- regulated profiles do not silently inherit casual defaults.

## 8. Milestone 5: Constitution Kernel

### M5.1 Deterministic policy enforcement

- `P0` Move beyond regex-like constitutional checks where needed.
- `P0` Ensure policy checks happen on execution requests, not only on prompts.
- `P0` Bind action classes to route context, evidence state, and risk.

Acceptance criteria:

- the model cannot bypass policy by phrasing,
- tool execution is gated by deterministic checks.

### M5.2 Approval and action classes

- `P0` Define approval thresholds by action class.
- `P1` Distinguish reversible from irreversible actions.
- `P1` Connect approval state to audit events and route traces.

Acceptance criteria:

- low-risk work is smooth,
- high-risk actions always trigger the expected control path.

### M5.3 Injection and content hardening

- `P0` Sanitize retrieved content before reinjection into model context.
- `P0` Separate source content from instruction-bearing system content.
- `P1` Add tests for indirect prompt injection and tool-output contamination.

Acceptance criteria:

- hostile retrieved content cannot silently rewrite execution policy,
- source data is handled as data, not trusted instructions.

## 9. Milestone 6: Operator Kernel

### M6.1 Route and evidence visibility

- `P1` Expose route traces.
- `P1` Expose evidence bundles and freshness states.
- `P1` Expose confidence movement across stages.

Acceptance criteria:

- operators can inspect why ORA answered the way it did,
- route and evidence state are legible without reading raw logs.

### M6.2 Mission and crawl visibility

- `P1` Expose mission status, artifacts, and failures.
- `P2` Add feed-style monitoring inspired by PulZ mission workflows.

Acceptance criteria:

- crawl operations are inspectable,
- artifacts can be reviewed and reused.

### M6.3 Review surfaces

- `P2` Add operator-facing summaries for contradiction cases.
- `P2` Add approval queue visibility.
- `P3` Add comparative route analytics.

Acceptance criteria:

- complex failures are reviewable without deep code inspection.

## 10. Milestone 7: Provider Resilience

### M7.1 Model abstraction

- `P1` Ensure the route kernel can target multiple providers cleanly.
- `P1` Add local fallback paths where practical.
- `P2` Add route policies for degraded-provider conditions.

Acceptance criteria:

- ORA can continue operating when a provider is unavailable or unsuitable,
- model choice does not rewrite the rest of the system.

### M7.2 Independence of core value

- `P1` Ensure memory, evidence, and route logic remain useful across providers.
- `P2` Avoid any design that makes ORA’s identity dependent on a single vendor-specific behavior.

Acceptance criteria:

- ORA’s main value survives provider changes.

## 11. Milestone 8: MCP and External Surface

### M8.1 MCP as delivery layer

- `P1` Expose ORA capabilities through a stable MCP interface.
- `P1` Separate external MCP tools from internal kernel stages.
- `P2` Add richer MCP resources once the internal path is real.

Acceptance criteria:

- MCP exposes the real ORA capability, not placeholder wrappers,
- external callers benefit from routing, memory, acquisition, and verification automatically.

### M8.2 Public capability map

- `P2` Define the primary externally visible capabilities, such as:
  - verified answer,
  - grounded summarize,
  - evidence bundle,
  - safe retrieval,
  - approval-gated action.

Acceptance criteria:

- the external interface is coherent and product-legible.

## 12. Testing and Benchmark Backlog

### T1 Core correctness

- `P0` Add route-kernel unit tests.
- `P0` Add retrieval normalization tests.
- `P0` Add verification behavior tests.
- `P0` Add constitution gating tests.

### T2 Integration

- `P0` Add end-to-end tests for:
  - memory-only answer,
  - local-doc answer,
  - live retrieval answer,
  - specialist route,
  - approval-gated action.

### T3 Security

- `P0` Add prompt injection and indirect prompt injection tests.
- `P1` Add evidence contamination tests.
- `P1` Add secret-boundary tests.

### T4 Performance

- `P1` Benchmark token spend by route class.
- `P1` Benchmark latency by route class.
- `P2` Benchmark memory hit-rate impact.
- `P2` Benchmark retrieval and crawl freshness.

Acceptance criteria for testing overall:

- no milestone is complete without direct tests for its critical behaviors,
- performance claims must be measured inside this repo.

## 13. Docs and Narrative Backlog

- `P0` Keep the master plan current.
- `P0` Keep the backlog current.
- `P1` Add a kernel map doc tied to concrete files.
- `P1` Add a route decision doc with examples.
- `P2` Add operator runbooks for crawl missions and audit review.

Acceptance criteria:

- docs reflect runtime reality,
- no primary doc implies delivery that the code does not provide.

## 14. Deferred Work

These are explicitly deferred until the core path works.

- `P3` advanced consensus mechanisms,
- `P3` elaborate multi-interface expansion,
- `P3` aggressive cryptographic branding work,
- `P3` high-concept swarm UI,
- `P3` speculative frontier integrations without route-kernel payoff.

## 15. First Recommended Implementation Sequence

If this backlog is approved, the next sequence should be:

1. milestone 0 reality reset,
2. milestone 1 route kernel,
3. milestone 2 memory kernel,
4. milestone 3 acquisition kernel,
5. milestone 4 verification kernel,
6. milestone 5 constitution hardening,
7. milestone 8 MCP surface cleanup,
8. milestone 6 operator visibility,
9. milestone 7 provider resilience tuning,
10. full benchmark and narrative lock.

## 16. Approval Checkpoint

Approval of this backlog means:

- this becomes the working execution list,
- the existing lightweight TODO is treated as obsolete or advisory,
- and the next working session should begin with milestone 0 plus milestone 1 file mapping.
