//! A read-only mirror of `ghlinks`'s `report.json` shape
//! (`ghlinks::model::Report` et al., as of `ghlinks` schema_version 2).
//!
//! This is a deliberate duplication, not a shared dependency: `ghlinks`
//! is a binary crate with no `lib.rs`, so there is nothing to import, and
//! even if there were, `ghbrief` is meant to be a genuinely separate
//! downstream tool per its own design brief — coupling it to `ghlinks`'s
//! internal types would blur that boundary. The tradeoff this accepts:
//! if `ghlinks` bumps `SCHEMA_VERSION`, this file needs a matching human
//! update. `check_schema_version()` below exists specifically so that
//! mismatch fails loudly and immediately instead of silently
//! misinterpreting a future, differently-shaped report.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The only `ghlinks` schema_version this file has been written against.
/// See `check_schema_version()`.
pub const SUPPORTED_SCHEMA_VERSION: u64 = 2;

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct ReleaseEntry {
    pub tag_name: Option<String>,
    pub name: Option<String>,
    pub published_at: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct RepoData {
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub license_key: Option<String>,
    pub license_name: Option<String>,
    pub stargazers_count: Option<i64>,
    pub forks_count: Option<i64>,
    pub watchers_count: Option<i64>,
    pub open_issues_count: Option<i64>,
    pub closed_issues_count: Option<i64>,
    pub primary_language: Option<String>,
    #[serde(default)]
    pub languages_bytes: BTreeMap<String, i64>,
    pub created_at: Option<String>,
    pub pushed_at: Option<String>,
    pub default_branch: Option<String>,
    pub commit_count_default_branch: Option<i64>,
    #[serde(default)]
    pub topics: Vec<String>,
    pub github_contributors_count: Option<i64>,
    #[serde(default)]
    pub github_contributors_count_semantics: String,
    pub releases_total_count: Option<i64>,
    pub releases_last_12_months: Option<i64>,
    pub latest_release_tag: Option<String>,
    pub latest_release_published_at: Option<String>,
    #[serde(default)]
    pub recent_releases: Vec<ReleaseEntry>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct GistData {
    pub description: Option<String>,
    pub owner_login: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub comments: Option<i64>,
    pub revision_count: Option<i64>,
    #[serde(default)]
    pub files: Vec<String>,
    pub note: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct ExternalMention {
    pub source: String,
    pub title: String,
    pub url: String,
    pub score: Option<i64>,
    pub num_comments: Option<i64>,
    pub created_at: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ExternalDiscovery {
    pub skipped: bool,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub coverage: String,
    #[serde(default)]
    pub hacker_news_query: String,
    #[serde(default)]
    pub hacker_news_status: String,
    #[serde(default)]
    pub hacker_news_mention_count: usize,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LinkRecord {
    pub input_url: String,
    pub canonical_url: Option<String>,
    pub link_kind: String,
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub file_path: Option<String>,
    pub repo_data: Option<RepoData>,
    pub gist_data: Option<GistData>,
    #[serde(default)]
    pub pages_candidates_checked: Vec<String>,
    pub pages_resolved_repo: Option<String>,
    #[serde(default)]
    pub external_mentions: Vec<ExternalMention>,
    pub external_discovery: ExternalDiscovery,
    #[serde(default)]
    pub fetch_errors: Vec<String>,
    #[serde(default)]
    pub fetched_at: String,
    #[serde(default)]
    pub collector_version: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RunSummary {
    #[serde(default)]
    pub ghlinks_version: String,
    #[serde(default)]
    pub github_api_version: String,
    #[serde(default)]
    pub hacker_news_api: String,
    #[serde(default)]
    pub reddit_note: String,
    #[serde(default)]
    pub started_at: String,
    #[serde(default)]
    pub finished_at: String,
    #[serde(default)]
    pub input_file: String,
    #[serde(default)]
    pub total_urls: usize,
    #[serde(default)]
    pub link_kind_counts: BTreeMap<String, usize>,
    #[serde(default)]
    pub records_with_errors: usize,
    #[serde(default)]
    pub concurrency: usize,
    #[serde(default)]
    pub delay_ms: u64,
    #[serde(default)]
    pub timeout_secs: u64,
    #[serde(default)]
    pub max_retries: u32,
    #[serde(default)]
    pub skip_external: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Report {
    pub schema_version: u64,
    pub run_summary: RunSummary,
    pub records: Vec<LinkRecord>,
}

/// Fails loudly on a schema this file wasn't written against, rather than
/// silently rendering a possibly-misleading brief from fields that may
/// have changed meaning or shape. Mirrors `ghlinks`'s own "honest failure
/// over silent best-effort" value — applied here at the boundary between
/// the two tools instead of within one of them.
pub fn check_schema_version(report: &Report) -> anyhow::Result<()> {
    if report.schema_version != SUPPORTED_SCHEMA_VERSION {
        anyhow::bail!(
            "report.json has schema_version {}, but ghbrief was written against \
             schema_version {}. Refusing to guess at a possibly-changed shape — \
             check ghlinks's ADRs/wrap-report-json-output-in-schema-versioned-envelope.md \
             for what changed, then update src/schema.rs to match before proceeding.",
            report.schema_version,
            SUPPORTED_SCHEMA_VERSION
        );
    }
    Ok(())
}
