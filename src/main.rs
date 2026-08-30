//! `ghbrief` — a deterministic, template-driven summarizer for
//! `ghlinks`'s `report.json`. No LLM, no network calls, no
//! nondeterminism: the same `report.json` plus the same config/templates
//! always produces byte-identical output.
//!
//! This is a genuinely separate downstream tool, not a `ghlinks`
//! feature — it depends only on `report.json`'s documented shape
//! (mirrored in `schema.rs`), never on `ghlinks`'s internals, and
//! `ghlinks` has no knowledge of this tool's existence. See
//! `schema::check_schema_version` for how that boundary is kept honest
//! as `ghlinks` evolves.
//!
//! Two independent ways to customize output without touching Rust code:
//! `--config` (thresholds and label wording — see `config.rs`) and
//! `--templates-dir` (sentence structure and overall tone — see
//! `templates/default/*.hbs`). Copy `templates/default/` to a new
//! directory and edit freely to build an alternate tone profile; nothing
//! about picking it up requires a code change.

mod config;
mod facts;
mod render;
mod schema;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

/// Deterministic, template-based summarizer for ghlinks report.json — no LLM required.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Path to a ghlinks report.json
    #[arg(long)]
    input: PathBuf,

    /// Where to write the rendered digest
    #[arg(long, default_value = "digest.md")]
    output: PathBuf,

    /// Directory containing the .hbs template files (see templates/default/ for the required set)
    #[arg(long, default_value = "templates/default")]
    templates_dir: PathBuf,

    /// TOML config overriding the embedded default thresholds/labels (see config/ghbrief.default.toml)
    #[arg(long)]
    config: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let raw = std::fs::read_to_string(&args.input)
        .with_context(|| format!("reading {}", args.input.display()))?;
    let report: schema::Report = serde_json::from_str(&raw)
        .with_context(|| format!("parsing {} as a ghlinks report.json", args.input.display()))?;
    schema::check_schema_version(&report)?;

    let cfg = config::Config::load(args.config.as_deref())?;
    eprintln!("ghbrief: config profile '{}'", cfg.tone.profile_name);
    let renderer = render::Renderer::load(&args.templates_dir)?;
    let now = chrono::Utc::now();

    let digest = render::render_report(&renderer, &report, &cfg, now)?;

    std::fs::write(&args.output, &digest)
        .with_context(|| format!("writing {}", args.output.display()))?;

    eprintln!(
        "ghbrief: wrote {} record(s) to {}",
        report.records.len(),
        args.output.display()
    );

    Ok(())
}
