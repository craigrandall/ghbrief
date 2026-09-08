# ghbrief Roadmap

This document outlines the planned evolution of ghbrief. Items are ordered by priority, not by timeline. Priorities may shift based on user feedback and real-world usage.

## High Priority

### Schema Version 3 Support
**Status:** Not started  
**Effort:** Medium  
**Why:** ghlinks may bump its schema version in the future. ghbrief needs to:
- Update `src/schema.rs` to mirror the new schema
- Update `SUPPORTED_SCHEMA_VERSION` constant
- Add any new fields to `RepoView` and other view models in `src/render.rs`
- Update tests to cover new schema fields

**Blockers:** None (waiting on ghlinks schema_version 3)

### Expand Test Coverage for Edge Cases
**Status:** Not started  
**Effort:** Medium  
**Why:** Current tests cover core functionality, but could benefit from:
- More boundary value tests in `facts.rs`
- Additional template rendering tests with edge case data
- Tests for malformed input handling

## Medium Priority

### Additional Bucketing Dimension: Repository Size
**Status:** Not started  
**Effort:** Medium  
**Why:** Users may want to categorize repos by size (small/medium/large) based on:
- Commit count
- Lines of code
- Repository size in bytes
- Number of files

This would follow the same pattern as existing bucketing dimensions (activity, popularity, release cadence).

**Considerations:** Requires ghlinks to collect the necessary data fields, or ghbrief to compute them from existing fields.

### Additional Template Variables
**Status:** Not started  
**Effort:** Low  
**Why:** ghlinks already collects some fields that ghbrief doesn't expose to templates:
- `created_at` (repo creation date) — already available, just not shown by default
- `default_branch`
- `commit_count_default_branch`
- `license_key` (SPDX identifier)

These could be added to `RepoView` in `src/render.rs` with minimal effort.

## Low Priority / Future Considerations

### Support for Custom Bucketing Dimensions via Config
**Status:** Not started  
**Effort:** High  
**Why:** Allow users to define their own bucketing criteria (e.g., "size", "team", "domain") via configuration, not just code changes.

**Complexity:** Requires significant design work to maintain determinism and type safety.

### Performance Optimization for Large Reports
**Status:** Not started  
**Effort:** Low-Medium  
**Why:** For reports with thousands of records, template rendering could be optimized:
- Parallel rendering of records
- Template compilation caching
- Streaming output instead of building in memory

**Note:** Premature optimization — wait for user reports of actual performance issues.

### Integration with Other Tools
**Status:** Not started  
**Effort:** Medium  
**Why:** Potential integrations:
- GitHub Actions for automated report generation
- Pre-commit hooks for local development
- Editor/IDE plugins for template editing

**Note:** Low priority until there is demonstrated user demand.

## Completed

### Initial Release (v0.1.2)
- Core summarization functionality
- Config-based customization (thresholds and labels)
- Template-based customization (structure and tone)
- Comprehensive documentation (README, CUSTOMIZING)
- Unit and integration tests

### Documentation Enhancements
- Added CUSTOMIZING.md with worked examples
- Added CONTRIBUTING.md
- Added ADRs for key architectural decisions
- Added ROADMAP.md

## How to Contribute

See `docs/CONTRIBUTING.md` for contribution guidelines. Items on this roadmap are open for contribution unless marked otherwise.

## Decision-Making

Priority and scope of items on this roadmap are determined by:
1. User demand and feedback
2. Alignment with project philosophy (deterministic, separate, honest)
3. Implementation effort vs. value
4. Maintainability and testing complexity

If you have a feature request or idea not listed here, please open a GitHub Issue for discussion.
