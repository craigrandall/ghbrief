//! Turns a `schema::Report` into rendered text. This module owns exactly
//! two responsibilities: (1) build a `serde`-serializable view model per
//! record — raw passthrough fields plus the labels `facts.rs` computed —
//! and (2) hand that to Handlebars. It contains no bucketing logic of its
//! own (that's `facts.rs`) and no CLI/file-I/O concerns (that's
//! `main.rs`).

use crate::config::Config;
use crate::facts;
use crate::schema::{LinkRecord, Report};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use handlebars::Handlebars;
use serde::Serialize;
use std::path::Path;

/// Template files `ghbrief` expects to find under a templates directory.
/// Deliberately a fixed, explicit list rather than "whatever files
/// happen to be in the directory" — a custom template profile that's
/// missing one of these fails with a clear "this file is missing" error
/// naming exactly what to add, rather than silently producing a thinner
/// digest than intended.
const KIND_TEMPLATES: &[&str] = &[
    "repo_root",
    "repo_file",
    "gist",
    "pages_site",
    "user_or_org_profile",
    "unsupported_github_url",
    "unknown",
    "fallback",
    "summary",
];
const PARTIAL_TEMPLATES: &[&str] = &[
    "partials/repo_stats",
    "partials/external_mentions",
    "partials/errors",
];

pub struct Renderer {
    hb: Handlebars<'static>,
}

impl Renderer {
    pub fn load(templates_dir: &Path) -> Result<Self> {
        let mut hb = Handlebars::new();
        // Catches a typo'd `{{field_name}}` in a custom template at
        // render time instead of it silently rendering as empty text —
        // every field this tool ever hands to a template is always
        // present (as `null` when unknown, never simply absent), so
        // strict mode should never misfire on legitimately-missing data.
        hb.set_strict_mode(true);

        for name in KIND_TEMPLATES.iter().chain(PARTIAL_TEMPLATES.iter()) {
            let path = templates_dir.join(format!("{name}.hbs"));
            hb.register_template_file(name, &path).with_context(|| {
                format!(
                    "loading required template '{name}.hbs' from {}",
                    templates_dir.display()
                )
            })?;
        }

        Ok(Renderer { hb })
    }

    fn render(&self, template: &str, data: &impl Serialize) -> Result<String> {
        self.hb
            .render(template, data)
            .with_context(|| format!("rendering template '{template}'"))
    }
}

#[derive(Serialize)]
struct ReleaseView {
    tag_name: Option<String>,
    published_at: Option<String>,
}

#[derive(Serialize)]
struct RepoView {
    description: Option<String>,
    homepage: Option<String>,
    license_name: Option<String>,
    stargazers_count: Option<i64>,
    forks_count: Option<i64>,
    open_issues_count: Option<i64>,
    closed_issues_count: Option<i64>,
    primary_language: Option<String>,
    top_languages: Vec<String>,
    pushed_at: Option<String>,
    created_at: Option<String>,
    default_branch: Option<String>,
    commit_count_default_branch: Option<i64>,
    topics: Vec<String>,
    contributors_count: Option<i64>,
    contributors_semantics: String,
    releases_total_count: Option<i64>,
    releases_last_12_months: Option<i64>,
    latest_release: Option<ReleaseView>,
    // Computed labels — the entire point of facts.rs existing.
    activity_label: String,
    popularity_label: String,
    release_cadence_label: String,
}

#[derive(Serialize)]
struct GistView {
    description: Option<String>,
    owner_login: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
    comments: Option<i64>,
    revision_count: Option<i64>,
    file_count: usize,
    files: Vec<String>,
    note: Option<String>,
}

#[derive(Serialize)]
struct ExternalView {
    skipped: bool,
    is_error: bool,
    mention_count: usize,
    top_mention_title: Option<String>,
    top_mention_url: Option<String>,
    top_mention_score: Option<i64>,
}

#[derive(Serialize)]
struct SummaryView<'a> {
    total_urls: usize,
    records_with_errors: usize,
    ghlinks_version: &'a str,
    started_at: &'a str,
    finished_at: &'a str,
    input_file: &'a str,
    link_kind_counts: &'a std::collections::BTreeMap<String, usize>,
}

const TOP_LANGUAGES_COUNT: usize = 3;

fn build_repo_view(repo: &crate::schema::RepoData, cfg: &Config, now: DateTime<Utc>) -> RepoView {
    let activity = facts::activity_bucket(repo.pushed_at.as_deref(), now, &cfg.activity);
    let popularity = facts::popularity_bucket(repo.stargazers_count, &cfg.popularity);
    let cadence = facts::release_cadence_bucket(repo.releases_last_12_months, &cfg.release_cadence);

    RepoView {
        description: repo.description.clone(),
        homepage: repo.homepage.clone(),
        license_name: repo.license_name.clone(),
        stargazers_count: repo.stargazers_count,
        forks_count: repo.forks_count,
        open_issues_count: repo.open_issues_count,
        closed_issues_count: repo.closed_issues_count,
        primary_language: repo.primary_language.clone(),
        top_languages: facts::top_languages(&repo.languages_bytes, TOP_LANGUAGES_COUNT),
        pushed_at: repo.pushed_at.clone(),
        created_at: repo.created_at.clone(),
        default_branch: repo.default_branch.clone(),
        commit_count_default_branch: repo.commit_count_default_branch,
        topics: repo.topics.clone(),
        contributors_count: repo.github_contributors_count,
        contributors_semantics: repo.github_contributors_count_semantics.clone(),
        releases_total_count: repo.releases_total_count,
        releases_last_12_months: repo.releases_last_12_months,
        latest_release: repo.latest_release_tag.as_ref().map(|tag| ReleaseView {
            tag_name: Some(tag.clone()),
            published_at: repo.latest_release_published_at.clone(),
        }),
        activity_label: activity.label(&cfg.activity).to_string(),
        popularity_label: popularity.label(&cfg.popularity).to_string(),
        release_cadence_label: cadence.label(&cfg.release_cadence).to_string(),
    }
}

fn build_gist_view(gist: &crate::schema::GistData) -> GistView {
    GistView {
        description: gist.description.clone(),
        owner_login: gist.owner_login.clone(),
        created_at: gist.created_at.clone(),
        updated_at: gist.updated_at.clone(),
        comments: gist.comments,
        revision_count: gist.revision_count,
        file_count: gist.files.len(),
        files: gist.files.clone(),
        note: gist.note.clone(),
    }
}

fn build_external_view(record: &LinkRecord) -> ExternalView {
    let top = record.external_mentions.first();
    ExternalView {
        skipped: record.external_discovery.skipped,
        // Handlebars has no built-in string-equality helper, so this
        // comparison happens here in Rust rather than in a template —
        // consistent with "facts.rs/render.rs decide, templates only
        // phrase" anyway.
        is_error: record.external_discovery.hacker_news_status == "error",
        mention_count: record.external_discovery.hacker_news_mention_count,
        top_mention_title: top.map(|m| m.title.clone()),
        top_mention_url: top.map(|m| m.url.clone()),
        top_mention_score: top.and_then(|m| m.score),
    }
}

/// Renders one record and returns the text block for it. Template
/// selection is a fixed lookup by `link_kind`, falling back to
/// `fallback.hbs` for any kind this file doesn't recognize — protecting
/// against a future `ghlinks` link_kind this copy of `ghbrief` predates,
/// per the same reasoning as `schema::check_schema_version`.
pub fn render_record(
    renderer: &Renderer,
    record: &LinkRecord,
    cfg: &Config,
    now: DateTime<Utc>,
) -> Result<String> {
    let template = match record.link_kind.as_str() {
        "repo_root"
        | "repo_file"
        | "gist"
        | "pages_site"
        | "user_or_org_profile"
        | "unsupported_github_url"
        | "unknown" => record.link_kind.as_str(),
        _ => "fallback",
    };

    #[derive(Serialize)]
    struct Ctx<'a> {
        input_url: &'a str,
        canonical_url: &'a Option<String>,
        owner: &'a Option<String>,
        repo: &'a Option<String>,
        file_path: &'a Option<String>,
        link_kind: &'a str,
        repo_data: Option<RepoView>,
        gist_data: Option<GistView>,
        pages_candidates_checked: &'a [String],
        pages_resolved_repo: &'a Option<String>,
        external: ExternalView,
        fetch_errors: &'a [String],
        fetched_at: &'a str,
    }

    let ctx = Ctx {
        input_url: &record.input_url,
        canonical_url: &record.canonical_url,
        owner: &record.owner,
        repo: &record.repo,
        file_path: &record.file_path,
        link_kind: &record.link_kind,
        repo_data: record
            .repo_data
            .as_ref()
            .map(|r| build_repo_view(r, cfg, now)),
        gist_data: record.gist_data.as_ref().map(build_gist_view),
        pages_candidates_checked: &record.pages_candidates_checked,
        pages_resolved_repo: &record.pages_resolved_repo,
        external: build_external_view(record),
        fetch_errors: &record.fetch_errors,
        fetched_at: &record.fetched_at,
    };

    renderer.render(template, &ctx)
}

/// Renders the whole report: a summary header followed by one block per
/// record, in input order (not re-sorted — `ghlinks`'s own record order
/// reflects input order too, and there's no reason to disturb that here).
pub fn render_report(
    renderer: &Renderer,
    report: &Report,
    cfg: &Config,
    now: DateTime<Utc>,
) -> Result<String> {
    let summary = SummaryView {
        total_urls: report.run_summary.total_urls,
        records_with_errors: report.run_summary.records_with_errors,
        ghlinks_version: &report.run_summary.ghlinks_version,
        started_at: &report.run_summary.started_at,
        finished_at: &report.run_summary.finished_at,
        input_file: &report.run_summary.input_file,
        link_kind_counts: &report.run_summary.link_kind_counts,
    };
    let mut out = renderer.render("summary", &summary)?;

    for record in &report.records {
        out.push('\n');
        out.push_str(&render_record(renderer, record, cfg, now)?);
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{ExternalDiscovery, ExternalMention, GistData, RepoData, RunSummary};
    use std::collections::BTreeMap;

    /// Loads the actual shipped default templates from disk — this is
    /// the closest thing to a compile check the Handlebars template
    /// files themselves get, since nothing else in this crate parses
    /// them. If a `.hbs` file has a syntax error or references a partial
    /// that was never registered, this is what catches it.
    fn test_renderer() -> Renderer {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("templates/default");
        Renderer::load(&dir).expect("shipped default templates must load without error")
    }

    fn test_config() -> Config {
        Config::embedded_default().unwrap()
    }

    fn minimal_external_discovery(skipped: bool, status: &str, count: usize) -> ExternalDiscovery {
        ExternalDiscovery {
            skipped,
            sources: vec![],
            coverage: "test".into(),
            hacker_news_query: "test".into(),
            hacker_news_status: status.into(),
            hacker_news_mention_count: count,
        }
    }

    fn minimal_record(link_kind: &str) -> LinkRecord {
        LinkRecord {
            input_url: "https://github.com/octocat/hello-world".into(),
            canonical_url: Some("https://github.com/octocat/hello-world".into()),
            link_kind: link_kind.into(),
            owner: Some("octocat".into()),
            repo: Some("hello-world".into()),
            file_path: None,
            repo_data: None,
            gist_data: None,
            pages_candidates_checked: vec![],
            pages_resolved_repo: None,
            external_mentions: vec![],
            external_discovery: minimal_external_discovery(true, "skipped", 0),
            fetch_errors: vec![],
            fetched_at: "2026-01-01T00:00:00Z".into(),
            collector_version: "0.0.0-test".into(),
        }
    }

    #[test]
    fn every_shipped_kind_template_renders_without_error_on_a_bare_record() {
        let renderer = test_renderer();
        let cfg = test_config();
        let now = Utc::now();
        // Every real link_kind ghlinks emits, rendered with nothing but
        // the minimal required fields — this is deliberately the sparsest
        // possible input for each kind, since that's exactly the shape
        // most likely to trip a template into referencing something
        // that isn't there.
        for kind in [
            "repo_root",
            "repo_file",
            "gist",
            "pages_site",
            "user_or_org_profile",
            "unsupported_github_url",
            "unknown",
            "a_future_kind_ghbrief_has_never_heard_of",
        ] {
            let record = minimal_record(kind);
            let rendered = render_record(&renderer, &record, &cfg, now)
                .unwrap_or_else(|e| panic!("rendering bare '{kind}' record failed: {e:?}"));
            assert!(!rendered.trim().is_empty(), "'{kind}' rendered to nothing");
        }
    }

    #[test]
    fn repo_root_with_full_data_surfaces_key_facts() {
        let renderer = test_renderer();
        let cfg = test_config();
        let now = Utc::now();

        let mut record = minimal_record("repo_root");
        let mut languages = BTreeMap::new();
        languages.insert("Rust".to_string(), 900_000);
        languages.insert("Shell".to_string(), 1_000);
        record.repo_data = Some(RepoData {
            description: Some("A deterministic link collector".into()),
            stargazers_count: Some(1234),
            forks_count: Some(56),
            open_issues_count: Some(3),
            closed_issues_count: Some(40),
            pushed_at: Some(now.to_rfc3339()),
            releases_last_12_months: Some(8),
            languages_bytes: languages,
            ..Default::default()
        });
        record.external_discovery = minimal_external_discovery(false, "ok", 1);
        record.external_mentions = vec![ExternalMention {
            source: "hacker_news".into(),
            title: "Show HN: a link collector".into(),
            url: "https://news.ycombinator.com/item?id=1".into(),
            score: Some(42),
            num_comments: Some(3),
            created_at: None,
        }];

        let rendered = render_record(&renderer, &record, &cfg, now).unwrap();

        assert!(rendered.contains("A deterministic link collector"));
        assert!(rendered.contains("1234"));
        assert!(rendered.contains("Rust"));
        assert!(
            rendered.contains(&cfg.activity.label_active),
            "a just-pushed repo should get the 'active' label"
        );
        assert!(rendered.contains(&cfg.release_cadence.label_frequent));
        assert!(rendered.contains("Show HN: a link collector"));
    }

    #[test]
    fn zero_values_are_shown_not_hidden_as_if_unknown() {
        // The whole point of `includeZero=true` throughout the repo_stats
        // partial: a confirmed zero must render, not disappear as if the
        // field were never fetched.
        let renderer = test_renderer();
        let cfg = test_config();
        let now = Utc::now();

        let mut record = minimal_record("repo_root");
        record.repo_data = Some(RepoData {
            stargazers_count: Some(0),
            open_issues_count: Some(0),
            releases_last_12_months: Some(0),
            ..Default::default()
        });

        let rendered = render_record(&renderer, &record, &cfg, now).unwrap();
        assert!(
            rendered.contains("Stars:** 0"),
            "confirmed-zero stars must render: {rendered}"
        );
        assert!(
            rendered.contains("0 open"),
            "confirmed-zero open issues must render: {rendered}"
        );
        assert!(
            rendered.contains(&cfg.release_cadence.label_none),
            "releases_last_12_months: Some(0) must map to the 'no recent releases' label, not 'unknown': {rendered}"
        );
    }

    #[test]
    fn missing_data_says_unknown_not_a_false_zero() {
        let renderer = test_renderer();
        let cfg = test_config();
        let now = Utc::now();

        // repo_data present but every bucketed field absent (None) —
        // distinct from the zero-values case above.
        let mut record = minimal_record("repo_root");
        record.repo_data = Some(RepoData::default());

        let rendered = render_record(&renderer, &record, &cfg, now).unwrap();
        assert!(rendered.contains(&cfg.activity.label_unknown));
        assert!(rendered.contains(&cfg.popularity.label_unknown));
        assert!(rendered.contains(&cfg.release_cadence.label_unknown));
        assert!(
            !rendered.contains("Stars:**"),
            "absent stargazers_count must not render a stats line at all"
        );
    }

    #[test]
    fn hacker_news_error_status_is_distinguished_from_zero_mentions() {
        let renderer = test_renderer();
        let cfg = test_config();
        let now = Utc::now();

        let mut record = minimal_record("repo_root");
        record.external_discovery = minimal_external_discovery(false, "error", 0);
        record.fetch_errors = vec!["hacker_news: request failed".into()];

        let rendered = render_record(&renderer, &record, &cfg, now).unwrap();
        assert!(rendered.contains("discovery failed"), "an HN error status must be visibly distinguished from a genuine zero-mentions result: {rendered}");
        assert!(
            rendered.contains("hacker_news: request failed"),
            "fetch_errors must always surface verbatim: {rendered}"
        );
    }

    #[test]
    fn gist_renders_file_list_and_counts() {
        let renderer = test_renderer();
        let cfg = test_config();
        let now = Utc::now();

        let mut record = minimal_record("gist");
        record.gist_data = Some(GistData {
            description: Some("A quick snippet".into()),
            files: vec!["main.rs".into(), "Cargo.toml".into()],
            comments: Some(0),
            ..Default::default()
        });

        let rendered = render_record(&renderer, &record, &cfg, now).unwrap();
        assert!(rendered.contains("A quick snippet"));
        assert!(rendered.contains("main.rs"));
        assert!(rendered.contains("Cargo.toml"));
        assert!(rendered.contains("Files:** 2"));
    }

    #[test]
    fn unresolved_pages_site_shows_candidates_and_errors_not_silence() {
        let renderer = test_renderer();
        let cfg = test_config();
        let now = Utc::now();

        let mut record = minimal_record("pages_site");
        record.pages_candidates_checked = vec![
            "octocat/octocat.github.io (not found)".into(),
            "octocat/hello-world (not found)".into(),
        ];
        record.fetch_errors = vec![
            "no candidate repo resolved automatically for this Pages site (...); tried octocat/octocat.github.io, octocat/hello-world — none of them resolved".into(),
        ];

        let rendered = render_record(&renderer, &record, &cfg, now).unwrap();
        assert!(rendered.contains("could not be automatically identified"));
        assert!(rendered.contains("octocat/octocat.github.io (not found)"));
        assert!(rendered.contains("none of them resolved"));
    }

    #[test]
    fn special_characters_in_source_data_are_never_html_escaped() {
        // Regression test: Handlebars' default `{{var}}` HTML-escapes
        // output (&, <, >, ", ', `, = all become entities), which is
        // correct for HTML but actively corrupts a Markdown digest.
        // Every template must use `{{{var}}}` (unescaped) for any value
        // that could contain these characters. A real ghlinks report
        // caught this: `github_contributors_count_semantics` contains a
        // literal `anon=true`, which rendered as `anon&#x3D;true` before
        // this was fixed.
        let renderer = test_renderer();
        let cfg = test_config();
        let now = Utc::now();

        let mut record = minimal_record("repo_root");
        record.repo_data = Some(RepoData {
            description: Some(
                r#"AT&T's "modern" <template> engine — anon=true & `escaped`?"#.into(),
            ),
            github_contributors_count_semantics: "anon=true; not unique humans".into(),
            github_contributors_count: Some(1),
            ..Default::default()
        });

        let rendered = render_record(&renderer, &record, &cfg, now).unwrap();
        assert!(
            rendered.contains(r#"AT&T's "modern" <template> engine — anon=true & `escaped`?"#),
            "description must appear byte-for-byte, not HTML-entity-encoded: {rendered}"
        );
        assert!(rendered.contains("anon=true; not unique humans"));
        for entity in [
            "&#x27;", "&quot;", "&#x3D;", "&amp;", "&#x60;", "&lt;", "&gt;",
        ] {
            assert!(
                !rendered.contains(entity),
                "found HTML entity '{entity}' in Markdown output: {rendered}"
            );
        }
    }

    #[test]
    fn full_report_renders_summary_followed_by_every_record() {
        let renderer = test_renderer();
        let cfg = test_config();
        let now = Utc::now();

        let mut link_kind_counts = BTreeMap::new();
        link_kind_counts.insert("repo_root".to_string(), 1usize);
        link_kind_counts.insert("gist".to_string(), 1usize);

        let report = Report {
            schema_version: crate::schema::SUPPORTED_SCHEMA_VERSION,
            run_summary: RunSummary {
                ghlinks_version: "0.14.9".into(),
                github_api_version: "2022-11-28".into(),
                hacker_news_api: "https://hn.algolia.com/api/v1/search".into(),
                reddit_note: "test".into(),
                started_at: "2026-01-01T00:00:00Z".into(),
                finished_at: "2026-01-01T00:01:00Z".into(),
                input_file: "links.txt".into(),
                total_urls: 2,
                link_kind_counts,
                records_with_errors: 0,
                concurrency: 4,
                delay_ms: 0,
                timeout_secs: 30,
                max_retries: 3,
                skip_external: false,
            },
            records: vec![minimal_record("repo_root"), minimal_record("gist")],
        };

        let digest = render_report(&renderer, &report, &cfg, now).unwrap();
        assert!(digest.contains("ghlinks Digest"));
        assert!(digest.contains("2 URL(s) processed"));
        assert!(digest.contains("octocat/hello-world"));
        // Two records means two "##" section headers beyond the summary's own.
        assert_eq!(digest.matches("\n## ").count(), 2);
    }
}
