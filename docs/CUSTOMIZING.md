# Customizing `ghbrief`'s digests: a guide for non-programmers

You don't need to know Rust to change how `ghbrief`'s digests look or
read. Almost everything about the *wording*, *thresholds*, and *layout*
of a digest lives in two kinds of plain text files you can open in any
text editor: a **config file** (numbers and short label words) and
**template files** (the sentence structure around those words). This
guide walks through real, worked examples of both, then — just as
importantly — draws an honest line around what those files *can't* do,
so you're not left guessing whether something is a five-minute edit or
not possible at all without a developer, or without an LLM instead of
`ghbrief` entirely.

If you only read one section, read "The four kinds of change" below —
everything else is worked examples of it.

## The four kinds of change

| If what you want is... | It's a... | Who/what can do it |
|---|---|---|
| Different **numbers or words** for the three existing labels (activity, popularity, release cadence) | **Config change** | You, editing a `.toml` file |
| Different **arrangement, selection, or phrasing** of facts `ghbrief` already collects | **Template change** | You, editing a `.hbs` file |
| A **new kind of fact** `ghbrief` doesn't compute yet, or **math/sorting/comparison across records** | **Code change** | A developer |
| Anything requiring **reading and understanding what text means** | **LLM needed** | Not `ghbrief` at all |

The first two are what this guide focuses on — they're genuinely yours
to make. The third and fourth are here so you can recognize them
quickly and stop looking for a text file to edit.

---

## Part 1 — Config changes: adjusting numbers and words

The config file is `config/ghbrief.default.toml`. **Don't edit that file
directly** — copy it somewhere (e.g. `my-config.toml`), edit your copy,
and tell `ghbrief` to use it:

```
ghbrief --input report.json --output digest.md --config my-config.toml
```

Every change below is made by opening your copy in Notepad (or any text
editor), changing a value, saving, and re-running the command above.

### Example 1.1 — "Active" feels too strict

By default, a repo counts as "actively maintained" only if it was pushed
to within the last 90 days. If that feels too strict for your purposes
(maybe you're fine with anything touched in the last 6 months counting
as active), change:

```toml
[activity]
active_within_days = 90
```
to:
```toml
[activity]
active_within_days = 180
```

**Before:** a repo pushed 120 days ago shows as *"maintained, though
pushes have slowed recently."*
**After:** the same repo now shows as *"actively maintained."* Nothing
else about the digest changes — this one number only affects that one
judgment call.

### Example 1.2 — Blunter, more casual wording

Maybe "not recently updated" feels too polite for how you actually think
about a project nobody's touched in 3 years. Change the label text
directly — the *threshold* for when this label applies doesn't have to
change at all:

```toml
label_dormant = "not recently updated"
```
to:
```toml
label_dormant = "looks abandoned"
```

**Before:** *"Status: not recently updated; small or niche; has not
published a release in the past 12 months."*
**After:** *"Status: looks abandoned; small or niche; has not published
a release in the past 12 months."*

Every other label works the same way — `label_active`,
`label_maintained`, `label_notable`, `label_frequent`, and so on are all
just text you can rewrite to match your own voice.

### Example 1.3 — Fewer repos get called "small or niche"

The popularity thresholds are also just numbers. If a lot of genuinely
useful repos you track are getting labeled "small or niche" because they
have, say, 15 stars, and you'd rather that only apply below 5:

```toml
[popularity]
emerging_at_stars = 10
```
to:
```toml
[popularity]
emerging_at_stars = 5
```

A repo with 8 stars now shows as *"gaining some traction"* instead of
*"small or niche."*

### The `_unknown` labels — leave these alone unless you mean it

Every bucket has a fourth label like `label_unknown = "activity could not
be determined (no push date available)"`. This isn't a bug or a
placeholder — it's what shows up when `ghlinks` genuinely couldn't
determine the value (as opposed to confirming it's zero). If you rename
`label_unknown` to sound the same as `label_dormant` or
`label_new_or_niche`, you'll lose a real, meaningful distinction: "we
don't know" and "we checked and the answer is no" are different facts,
and `ghbrief` goes out of its way to keep them apart. Reword these if you
like, but keep them saying something distinct from the other three.

---

## Part 2 — Template changes: reshaping what's said and how

Templates live in `templates/default/*.hbs` — copy the whole
`templates/default` folder (e.g. to `templates/mine`), edit your copy,
and point `ghbrief` at it:

```
ghbrief --input report.json --output digest.md --templates-dir templates/mine
```

**Which file to edit** depends on what kind of link you're changing the
write-up for:

| To change how this shows up... | Edit this file |
|---|---|
| A regular repository | `repo_root.hbs` |
| A link to one file inside a repository | `repo_file.hbs` |
| A GitHub Gist | `gist.hbs` |
| A GitHub Pages site | `pages_site.hbs` |
| The stats block shared by repos and resolved Pages sites | `partials/repo_stats.hbs` |
| The Hacker News mention line | `partials/external_mentions.hbs` |
| The list of problems/notes at the bottom of a record | `partials/errors.hbs` |
| The header at the very top of the digest | `summary.hbs` |

Most of the interesting editing happens in `partials/repo_stats.hbs`,
since it's shared by the most common record type.

### Example 2.1 — Hide the Topics line

Some digests feel cluttered by a long topics list. Find this block in
`partials/repo_stats.hbs`:

```
{{#if topics}}- **Topics:** {{#each topics}}{{{this}}}{{#unless @last}}, {{/unless}}{{/each}}
{{/if}}
```

Delete those two lines entirely. That's the whole change — nothing else
needs to move.

### Example 2.2 — Show when the repo was created (data you already have!)

`ghlinks` already collects each repo's creation date — `ghbrief` just
doesn't print it by default. Add a line anywhere in
`partials/repo_stats.hbs`, right next to the similar "Last pushed" line:

```
{{#if created_at}}- **Created:** {{{created_at}}}
{{/if}}
```

This is a good example of the cheapest kind of customization: the fact
was already sitting there, unused — you're just choosing to show it.
(`default_branch` is another field like this, already collected and
available, just not shown by default.)

### Example 2.3 — Reorder facts

Want "Last pushed" to appear right after the description, before stars?
Just cut that block and paste it earlier in the file. Handlebars doesn't
care about order beyond the order you physically write the lines in —
there's no separate "ordering" setting to find.

### Example 2.4 — Avoid double-quote clutter in Hacker News mentions

If a Hacker News story title itself contains quotation marks, wrapping
it in more quotes can look cluttered (e.g. `"Why Git is no "good" for
AI-generated code""`). In `partials/external_mentions.hbs`, change:

```
e.g. "{{{top_mention_title}}}"
```
to:
```
e.g. _{{{top_mention_title}}}_
```

Same information, shown in italics instead of quotes — no nesting
problem, and it reads more naturally either way.

### Example 2.5 — Label section headers by kind

If you're skimming a long digest and want repo sections visually
distinct from Gist sections at a glance, change the heading in
`repo_root.hbs` from:

```
## {{{owner}}}/{{{repo}}}
```
to:
```
## [Repo] {{{owner}}}/{{{repo}}}
```

...and similarly add `[Gist]`, `[Pages]`, etc. to the other files' headers.

### One technical detail worth knowing before you add a new line

If you add a line referencing a number that could legitimately be `0`
(stars, issues, a count of any kind), write the check as
`{{#if some_number includeZero=true}}` rather than plain `{{#if
some_number}}`. Without `includeZero=true`, a genuine "0" is treated the
same as "we don't have this data at all" and the line silently
disappears — which is exactly the "don't blur zero with unknown"
principle this project cares about throughout. Copy the pattern from any
existing numeric line in `repo_stats.hbs` if you're unsure.

---

## Part 3 — What needs a developer (not just a text edit)

These are real, verified examples of things `ghbrief`'s data or code
doesn't currently support — no amount of config or template editing
gets you there, because the underlying capability isn't built yet:

- **Showing full release history, not just the latest release.**
  `ghlinks` collects up to 100 recent releases per repo
  (`recent_releases`), but `ghbrief` currently only ever hands the
  *latest one* to templates. Showing the rest requires a small Rust
  change to pass the full list through.
- **Showing the GitHub-native license identifier (e.g. `MIT`) alongside
  the human-readable name (e.g. `MIT License`).** `ghlinks` collects
  both, but `ghbrief` currently only wires up the human-readable one.
- **Sorting or ranking records** — e.g., "list the most-starred repos
  first," or "put unresolved Pages sites at the top so I see problems
  first." `ghbrief` currently just prints records in whatever order
  `ghlinks` originally wrote them; there's no sorting step to configure,
  and Handlebars templates can't sort a list themselves.
- **Report-wide statistics** — e.g., "show the average star count across
  all 98 repos," or "show total combined stars." The digest header
  currently only shows counts (how many URLs, how many errors, how many
  of each kind) — not sums or averages across records.
- **A brand new bucket dimension** — e.g., categorizing repos by size
  (small/medium/large) based on commit count, the way activity and
  popularity are already categorized. The *pattern* exists in the code
  and a developer could copy it, but there's no config setting that
  invents a new dimension from nothing.

If you want any of these, that's a good, concrete thing to hand to
whoever maintains the `ghbrief` codebase — each one is a small, bounded
change, just not a text-file one.

---

## Part 4 — What needs an LLM, not `ghbrief` at all

This is the most important boundary to recognize, because it's tempting
to keep tweaking config or templates hoping to get there and never
quite arriving. **`ghbrief` can relabel, reorder, and restyle facts it
already has. It cannot read anything and decide what it means.** That
second thing was always meant to be a separate, later stage in the
pipeline — the whole reason `ghlinks` produces a plain, honest
`report.json` in the first place is so an LLM can pick up exactly this
kind of task afterward. Signs you've hit this boundary:

- **"Summarize what this repo does" in fewer/different words than its
  own description.** `ghbrief` can only show you the description
  `ghlinks` already collected, verbatim — it can't compress, rephrase,
  or improve on it.
- **Sentiment or tone characterization** — e.g., "was the Hacker News
  discussion positive or critical about this project?" `ghbrief` can
  tell you *how many* mentions there were and show you the top one's
  title; it has no ability to judge what people in that discussion
  actually thought.
- **Comparative or evaluative judgment across records** — "which of
  these 40 repos seems most promising," "which two of these look like
  they're duplicates of each other," "rank these by how mature they
  seem." Anything that requires weighing multiple records against each
  other in a way that isn't a plain sort-by-a-number is a judgment call,
  not a lookup.
- **Categorizing by theme or topic that isn't already a discrete field**
  — e.g., "flag anything that looks AI-agent-related." A crude version
  of this (matching a fixed list of keywords against the description) is
  technically a *code* change, not an LLM one — but it will miss
  paraphrases and catch false positives in ways a genuine reading of the
  text wouldn't. If you want this done *well*, it's an LLM task; if a
  rough pass is good enough, it's worth asking a developer whether a
  simple keyword match is worth adding.
- **Filling in gaps `ghlinks` honestly couldn't close** — the clearest
  example already exists in `ghlinks` itself: when a GitHub Pages site's
  backing repository can't be automatically identified, its error
  message explicitly suggests "a web search for the site's repository"
  as the next step. That's not a `ghbrief` job by design — it's exactly
  the kind of thing meant for an LLM stage with web search, picking up
  precisely where the deterministic tools stop.
- **Wanting each write-up to feel individually composed** rather than
  visibly following the same sentence pattern every time. Templates
  produce the same shape on purpose — that's what makes them
  deterministic and testable — but it does mean 40 repo write-ups will
  all read with a similar rhythm. If you want prose that varies the way
  a person's writing naturally would, that variability is an LLM trait,
  not a template one.

## The decision aid, restated

- Different **words or numbers** for a label that already exists →
  **config**.
- Different **selection, order, or structure** of facts already being
  collected → **template**.
- A **new fact, or math/sorting/comparison across records** → **ask a
  developer**.
- Anything that requires **reading and understanding** what text means →
  **that's an LLM's job**, and it always was — `ghbrief` was built to
  make that step optional for the cases where relabeling known facts is
  good enough, not to replace it.
