---
status: "accepted"
date: 2026-09-08
decision-makers: Craig
consulted: clankers
informed: n/a — single-developer project
---

# ghbrief must be fully deterministic

## Context and Problem Statement

When designing ghbrief, a fundamental question arose: should the summarization tool be deterministic (same input always produces same output) or allow for probabilistic/ML-based synthesis?

This decision affects:
- Reproducibility of results
- Testability and verification approach
- Cost and complexity of running the tool
- Risk of hallucinations or errors in output
- User trust in the generated summaries

## Decision Drivers

- **Reproducibility** – Users need to trust that the same `report.json` + config + templates always produces identical output.
- **Testability** – Deterministic behavior enables comprehensive unit and integration testing.
- **Zero cost** – No LLM API calls means no operational cost for users.
- **Zero hallucination risk** – No ML means no fabricated information in summaries.
- **Honest failure visibility** – Aligns with ghlinks's core value of explicit, honest error reporting.

## Considered Options

- **Option A: Fully deterministic (chosen)** – No LLM, no network calls, no randomness. Pure functions and template rendering only.
- **Option B: Hybrid approach** – Deterministic for known facts, LLM for synthesis (e.g., cross-record analysis, description summarization).
- **Option C: LLM-first** – Use LLM for all summarization, with deterministic data as context.
- **Option D: Configurable** – Let users choose between deterministic and LLM-based modes.

## Decision Outcome

Chosen option: **"Option A: Fully deterministic"**, because it aligns with the project's core philosophy of honest, reproducible processing. It costs nothing to run, has zero hallucination risk, and is exactly as testable as ghlinks itself — every bucketing decision lives in a pure function in `src/facts.rs`, unit-tested the same way ghlinks's `retry.rs` is.

### Consequences

- Good, because output is 100% reproducible — same input always yields same output.
- Good, because zero operational cost — no API keys, no rate limits, no billing.
- Good, because zero hallucination risk — no fabricated information.
- Good, because fully testable — every decision path can be unit-tested.
- Good, because aligns with ghlinks's "honest failure visibility" value.
- Bad, because cannot perform synthesis that requires understanding (e.g., cross-record analysis, description rephrasing).
- Bad, because users who want LLM-based summarization must use a separate tool in their pipeline.

### Confirmation

This decision is confirmed by the implementation:
- All bucketing logic in `src/facts.rs` consists of pure functions (no I/O, no global state).
- Template rendering in `src/render.rs` uses only data passed explicitly to templates.
- The `main.rs` CLI passes a timestamp (`now`) explicitly to bucketing functions rather than reading a clock internally, preserving determinism.
- The README explicitly states: "No LLM, no network calls, no nondeterminism: the same `report.json` plus the same config and templates always produces byte-identical output."

## Pros and Cons of the Options

### Option A: Fully deterministic

- Good, because aligns with the division of labor philosophy — deterministic collection (ghlinks) + deterministic summarization (ghbrief), with any probabilistic synthesis pushed to a separate downstream stage.
- Good, because enables the same verification approach as ghlinks — pure functions, unit tests, no mocking required.
- Good, because zero operational complexity — no API management, no rate limiting, no authentication.
- Neutral, because limits functionality to relabeling and restructuring known facts, but this is the intended scope.

### Option B: Hybrid approach

- Good, because could provide richer summaries for users who want cross-record synthesis.
- Bad, because introduces non-determinism for a subset of functionality, complicating the mental model.
- Bad, because requires LLM integration, API keys, rate limiting, and cost management.
- Bad, because blurs the division of labor — deterministic collection vs. probabilistic synthesis.

### Option C: LLM-first

- Good, because could produce more natural, varied output.
- Bad, because completely undermines the deterministic philosophy.
- Bad, because introduces all LLM complexities (cost, hallucinations, non-determinism).
- Bad, because cannot be unit-tested in the same way.

### Option D: Configurable

- Good, because gives users choice.
- Bad, because increases complexity for both implementation and user understanding.
- Bad, because the "deterministic mode" would likely become a second-class citizen.
- Bad, because defeats the purpose of a clear, focused tool.

## More Information

See `ghbrief/README.md` sections "Why deterministic, and why separate?" and "Known limitations (by design, not oversights)" for additional context.

See `docs/CUSTOMIZING.md` for the explicit boundaries between what ghbrief can do (config and template changes) and what requires an LLM (reading and understanding text meaning).
