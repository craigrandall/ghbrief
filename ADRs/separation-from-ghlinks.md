---
status: "accepted"
date: 2026-09-08
decision-makers: Craig
consulted: clankers
informed: n/a — single-developer project
---

# Keep ghbrief as a separate tool from ghlinks

## Context and Problem Statement

`ghlinks` is a deterministic collector for GitHub-hosted links that produces a structured `report.json` file. A natural question arises: should summarization functionality be built into `ghlinks` itself (as a subcommand or output format option), or should it be a separate downstream tool?

This decision affects:
- Coupling between collection and presentation logic
- Evolution velocity of each component
- Testing complexity and surface area
- Deployment and distribution model

## Decision Drivers

- **Separation of concerns** – Collection (ghlinks) and presentation (ghbrief) are distinct responsibilities that should not be interleaved.
- **Independent evolution** – Each tool should evolve at its own pace without requiring coordinated releases.
- **Zero dependency on ghlinks internals** – ghbrief should depend only on the documented `report.json` schema, not on ghlinks's internal types or implementation.
- **Honest failure visibility** – Both tools share this value; keeping them separate prevents one from compromising the other's error handling.

## Considered Options

- **Option A: Separate binary/repo (chosen)** – ghbrief is a standalone Rust binary in its own repository, reading `report.json` as input.
- **Option B: ghlinks subcommand** – Add `ghlinks brief` as a subcommand that performs summarization.
- **Option C: ghlinks library dependency** – Extract shared types from ghlinks and have ghbrief depend on a `ghlinks-core` library.
- **Option D: Shared library with feature flags** – Single binary with feature flags to enable/disable collection vs. summarization.

## Decision Outcome

Chosen option: **"Option A: Separate binary/repo"**, because it best preserves the separation of concerns and allows completely independent evolution. The collector (ghlinks) remains free of any output-formatting concerns, and ghbrief (or any future summarizer) can be swapped, evolved, or replaced without either tool needing to know about the change.

### Consequences

- Good, because ghlinks stays focused on honest fact-gathering with explicit failure reporting.
- Good, because ghbrief can evolve its summarization approach (templates, config, bucketing logic) without requiring ghlinks releases.
- Good, because users can choose alternative summarizers (including LLM-based ones) without affecting ghlinks.
- Good, because testing surface area is minimized — each tool tests its own responsibilities.
- Bad, because users must install and manage two separate tools.
- Bad, because schema version coordination must be manual (mitigated by explicit version checking in ghbrief).

### Confirmation

This decision is confirmed by the implementation: ghbrief contains a manual mirror of ghlinks's `report.json` schema in `src/schema.rs` (not generated from ghlinks source), and `check_schema_version()` in that same file will refuse to run if the schema version doesn't match what ghbrief was written against. This ensures the boundary between the two tools remains honest and explicit.

## Pros and Cons of the Options

### Option A: Separate binary/repo

- Good, because cleanest separation of concerns — collection vs. presentation are distinct stages in a pipeline.
- Good, because each tool has a single, focused responsibility.
- Good, because independent release cycles — ghlinks can add new data fields without breaking ghbrief (as long as schema version is bumped).
- Good, because different deployment models — users who only need collection don't pay the cost of summarization dependencies.
- Neutral, because requires users to manage two tools, but this is acceptable for a pipeline architecture.

### Option B: ghlinks subcommand

- Good, because single installation for users who want both collection and summarization.
- Bad, because couples presentation logic to the collector, increasing ghlinks's complexity and testing surface.
- Bad, because ghlinks would need to know about summarization concerns, blurring the division of labor.
- Bad, because alternative summarizers would need to be subcommands too, or users would be locked into ghlinks's built-in approach.

### Option C: ghlinks library dependency

- Good, because shared types would be automatically in sync.
- Bad, because ghlinks is a binary crate with no `lib.rs`, so there is nothing to import — this option is not technically feasible without restructuring ghlinks.
- Bad, because even if feasible, it would couple ghbrief to ghlinks's internal types, making independent evolution difficult.

### Option D: Shared library with feature flags

- Good, because single binary for users.
- Bad, because increases binary size for users who only need one function.
- Bad, because couples the evolution of collection and presentation logic.
- Bad, because violates the separation of concerns principle that both tools were built around.

## More Information

See `ghbrief/README.md` section "Why deterministic, and why separate?" for additional rationale.
