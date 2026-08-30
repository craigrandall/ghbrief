# ghbrief

A deterministic, template-driven summarizer for [`ghlinks`](https://github.com/craigrandall/ghlinks)'s `report.json`. No LLM, no network calls, no nondeterminism: the same `report.json` plus the same config and templates always produces byte-identical output.

This is a genuinely separate downstream tool, not a `ghlinks` feature. It depends only on the documented shape of `report.json` (schema_version 2) — never on `ghlinks`'s internals — and `ghlinks` has no knowledge that `ghbrief` exists. See "Why deterministic, and why separate?" below.

## Usage

```
ghbrief --input report.json --output digest.md
```

Optional flags:
- `--templates-dir <dir>` — defaults to `templates/default`. Point this at a copy you've customized (see "Customizing tone" below).
- `--config <file>` — defaults to the embedded default thresholds (`config/ghbrief.default.toml`). Point this at your own copy to change what counts as "active," "notable," etc.

## What it produces

One section per `ghlinks` record, in the order `ghlinks` wrote them, preceded by a short run-summary header. Example, for a `repo_root` record:

```
## octocat/hello-world
https://github.com/octocat/hello-world

> My first repository on GitHub!
- **Status:** actively maintained; widely used; releases regularly.
- **Stars:** 2847 · **Forks:** 1023
- **Issues:** 12 open, 340 closed
- **Languages:** Ruby, JavaScript
- **Last pushed:** 2026-08-20T14:03:00Z
- **Contributors:** 8 (counted via GitHub's contributors API; co-authors may be over/undercounted per GitHub's own rules)
- **Latest release:** v2.1.0 (2026-07-01T00:00:00Z)
```

Records with `fetch_errors` always show them verbatim under a **Notes** line — `ghbrief` never summarizes an error away or omits it for brevity, matching `ghlinks`'s own "honest failure visibility" value.

## Two independent ways to customize, without touching Rust code

**Thresholds and label wording** (`--config`): what counts as "active" (days since last push), "notable" (star count), or "releases regularly" (releases in the last 12 months), and the exact words used for each bucket. Copy `config/ghbrief.default.toml`, edit the numbers and/or words, pass it via `--config`.

**Sentence structure and tone** (`--templates-dir`): copy `templates/default/` to e.g. `templates/casual/`, edit the `.hbs` files freely — reorder facts, change punctuation, drop a section entirely, restructure the prose — and point `--templates-dir` at your copy. No recompilation needed; templates are read from disk at runtime.

These two axes are intentionally independent: you can make the tone more casual without touching a single threshold, or tighten what counts as "active" without touching a word of prose.

**New to editing config/templates, or not a programmer?** See [`docs/CUSTOMIZING.md`](docs/CUSTOMIZING.md) — a worked-example guide covering both, plus an explicit list of changes that need a developer or an LLM instead.

### Required template files

Every `--templates-dir` must contain these files (a fixed, explicit list — `ghbrief` fails with a clear "missing file" error naming exactly which one, rather than silently producing a thinner digest):

```
repo_root.hbs
repo_file.hbs
gist.hbs
pages_site.hbs
user_or_org_profile.hbs
unsupported_github_url.hbs
unknown.hbs
fallback.hbs          # any link_kind this copy of ghbrief doesn't recognize
summary.hbs           # the run-summary header
partials/repo_stats.hbs
partials/external_mentions.hbs
partials/errors.hbs
```

Each record's context includes every field `ghlinks` writes for that record (see `src/schema.rs` and `src/render.rs` for the exact shape passed to each template), plus three computed labels on `repo_data` — `activity_label`, `popularity_label`, `release_cadence_label` — driven by `--config`.

One Handlebars detail worth knowing if you edit templates: fields that can legitimately be a confirmed `0` (stars, open issues, releases in the last 12 months, etc.) use `{{#if field includeZero=true}}` rather than bare `{{#if field}}` — plain `{{#if}}` treats `0` as falsy, which would wrongly hide a real "0 open issues" the same way it hides a genuinely-missing field. If you add a new numeric field to a template, carry this forward.

## Why deterministic, and why separate?

`ghlinks` was built around a specific division of labor: a deterministic collector at the front of a pipeline, doing honest fact-gathering with explicit failure reporting, with any probabilistic synthesis pushed to a separate downstream stage. `ghbrief` extends that same division one stage further rather than blurring it — it's Option A of a broader set of options considered for "how do I get a useful summary out of `report.json`" (template-only, at one end, vs. increasingly LLM-dependent options at the other). It costs nothing, has zero hallucination risk, and is exactly as testable as `ghlinks` itself: every bucketing decision lives in a pure function in `src/facts.rs`, unit-tested the same way `ghlinks`'s `retry.rs` is.

It intentionally stays a separate binary/repo rather than becoming a `ghlinks` subcommand or library dependency — the collector stays free of any output-formatting concerns, and `ghbrief` (or a future richer summarizer) can evolve, or be swapped for something else entirely, without either tool needing to know about the change.

## Known limitations (by design, not oversights)

- **No synthesis across records.** Each record is summarized independently; there's no "here's the common thread across these 40 repos" — that's exactly the kind of task an LLM stage does better, and is deliberately out of scope here.
- **`activity_label` reflects `pushed_at` recency *at the time `ghbrief` runs*, not at collection time.** A `report.json` read a year after it was generated will show a much more "dormant"-leaning activity label than the day it was collected, even though nothing about the repo's data changed — the underlying `pushed_at` value is fixed, but "how long ago was that" isn't.
- **Every `_unknown` label exists on purpose.** A missing `pushed_at`/`stargazers_count`/`releases_last_12_months` is not evidence of "none" — see `src/facts.rs`'s module doc-comment. If a custom template collapses these into the same wording as a real zero, that's introducing exactly the ambiguity `ghlinks` itself works hard to avoid.
- **Schema coupling is explicit, not implicit.** `src/schema.rs` mirrors `ghlinks` report.json's shape as of schema_version 2 by hand; it isn't generated from `ghlinks`'s source. If `ghlinks` ever bumps `schema_version`, `check_schema_version()` will refuse to run rather than silently misinterpret a changed shape — at which point `src/schema.rs` (and likely `render.rs`) need a matching update.

## Development

```
cargo fmt && cargo check && cargo clippy && cargo test && cargo build --release
```

Unit tests: `src/facts.rs` (pure bucketing logic — boundary values, `None` vs. confirmed-zero, malformed/future dates). Integration-style tests: `src/render.rs` (loads the real shipped templates from disk and renders synthetic records through them — the closest thing to a compile check the `.hbs` files themselves get, since nothing else parses them).

---

## Contributing

Contributions are welcome. See `docs\CONTRIBUTING.md` for more details.

## License

Dual-licensed under MIT or Apache-2.0, at your option. See `LICENSE-MIT`
and `LICENSE-APACHE`.
