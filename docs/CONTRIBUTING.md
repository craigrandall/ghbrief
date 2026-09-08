# ghbrief Contributor Guide

## 1. Getting Started

**Clone and build:**

```bash
git clone https://github.com/craigrandall/ghbrief.git
cd ghbrief
cargo build --release
```

ghbrief is a deterministic, template-driven summarizer for `ghlinks`'s `report.json`. It takes structured data collected by ghlinks and produces human-readable markdown digests.

## 2. Project Structure

- **`src/main.rs`** – CLI entry point, argument parsing, and orchestration. `main()` is minimal; core logic lives in other modules.
- **`src/schema.rs`** – Data structures mirroring ghlinks's `report.json` shape (schema_version 2). Contains `check_schema_version()` to ensure compatibility.
- **`src/facts.rs`** – Pure bucketing logic for activity, popularity, and release cadence. No I/O, fully unit-tested.
- **`src/render.rs`** – Template rendering with Handlebars. Builds view models from schema data and renders via templates.
- **`src/config.rs`** – Configuration loading (thresholds and labels). Supports embedded defaults and external TOML files.
- **`config/ghbrief.default.toml`** – Embedded default configuration.
- **`templates/default/`** – Default Handlebars templates for all record types.

## 3. Development Workflow

1. **Pick an issue or feature** to work on (or propose one via GitHub Issues).

2. **Create a branch**:

   ```bash
   git checkout -b feature/short-description
   ```

3. **Implement changes** with:

   - Clear separation of concerns (see module responsibilities below).
   - Strong typing and explicit error handling.
   - Deterministic behavior — no randomness, no network calls, no LLM dependencies.

4. **Run the verification sequence**:

   ```bash
   cargo fmt && cargo check && cargo clippy && cargo test && cargo build --release
   ```

   This is the **same sequence the CI pipeline runs**. All five commands must pass.

5. **Submit a pull request** with:

   - Clear description of changes.
   - Rationale for the approach.
   - Notes on any new configuration options or schema impacts.

## 4. Coding Standards

- Use idiomatic Rust (`?` for error propagation, `Result<T, E>`, `Option<T>`).
- Prefer small, composable, pure functions (see `facts.rs` for the model example).
- Avoid `unsafe` — there is no valid use case for it in this project.
- Keep module boundaries clean:
    - `schema` – data structures only, no logic.
    - `facts` – pure bucketing logic only, no I/O.
    - `render` – template rendering only, no bucketing.
    - `config` – configuration loading only.
    - `main` – orchestration only, no business logic.

## 5. Error Handling

- Use `anyhow::Context` for high-level error context.
- Ensure errors are descriptive and actionable.
- Distinguish between "missing data" (`None`) and "confirmed zero" (`Some(0)`) — this is a core project value.

## 6. Testing

**Unit tests:**

- `facts.rs` – Bucketing logic (activity, popularity, release cadence). Test boundary values, `None` vs. confirmed-zero, malformed/future dates.
- `render.rs` – Template rendering. Loads actual shipped templates from disk and renders synthetic records through them.

**Test patterns to follow:**

- Keep tests isolated and deterministic.
- Test edge cases, not just happy paths.
- Use `includeZero=true` in Handlebars templates to preserve the distinction between 0 and missing.

## 7. Documentation

ghbrief keeps most implementation detail in source doc-comments (`//!`) rather than separate design documents. Update documentation in these locations:

- **Source code `//!` comments** – Authoritative for behavior. Update when:
    - Changing externally observable behavior.
    - Adding non-obvious design decisions.

- **README.md** – Update when:
    - Adding new CLI flags or configuration options.
    - Changing output format or schema expectations.
    - Adding or changing "Known limitations."

- **CONTRIBUTING.md** – Update when:
    - Changing the development workflow or verification sequence.

- **ADRs/** – Record new Architecture Decision Records for decisions that are:
    - Expensive or disruptive to reverse.
    - Constrain later work across components.
    - Had genuine alternatives a reasonable engineer would weigh.

If documents disagree, **source code and its `//!` comments are authoritative for behavior**, ADRs are authoritative for *why* a decision was made, and README/CONTRIBUTING are authoritative for user/contributor-facing summaries. Fix summaries to match the source, not vice versa.

## 8. AI-Assisted Review & Verification

Some contributions to this project — including drafts of source code,
ADRs, and documentation — are produced or reviewed with the help of an AI
assistant. That assistant frequently runs in a sandboxed environment with
**no Rust toolchain and no network access**, meaning any code it produces
or reviews has been read for correctness but has **not** been compiled,
linted, or executed. Treat AI-assisted output as unverified by
construction, regardless of how confident or polished it looks, until it
clears the same gate as any other change.

The actual verification gate for this project is local and manual. Before
considering any change — AI-assisted or not — ready for a pull request,
run all of the following, in order, and confirm each one is clean:

```bash
cargo fmt
cargo check
cargo clippy
cargo test
cargo build --release
```

A pull request that includes AI-assisted changes should say so, and
should confirm this sequence was actually run locally rather than
inferred from the code "looking right." If any step above hasn't been
run, say that explicitly in the PR rather than implying it has — this
project would rather show an honest "not yet verified" than a false
green.

## 9. Continuous Integration

The project uses GitHub Actions for automated verification:

### CI Workflow

The CI workflow (`.github/workflows/ci.yml`) runs automatically on:
- Every push to the `main` branch
- Every pull request targeting the `main` branch

It executes the complete verification gate in sequence:

1. **`cargo fmt --check`** - Verifies code formatting
2. **`cargo clippy -- -D warnings`** - Lints with warnings as errors
3. **`cargo check`** - Checks compilation
4. **`cargo test`** - Runs all tests
5. **`cargo build --release`** - Builds the release binary

**If any step fails, the workflow fails and the PR cannot be merged.**

### Release Workflow

This has not yet been implemented since the project is not yet in a release-worthy state.

### Local Verification

Before pushing changes, run the verification gate locally:

```bash
cargo fmt
cargo clippy
cargo check
cargo test
cargo build --release
```

This matches exactly what the CI workflow will run, allowing you to catch issues before they reach CI.
