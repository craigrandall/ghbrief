//! Deterministic bucketing thresholds and their labels, loaded from a
//! TOML file. This is one of two independent ways `ghbrief` is meant to
//! be tweaked without touching Rust code — see `templates/` for the
//! other (sentence structure/tone). This file controls *what counts as*
//! "active" or "notable"; the templates control *how that's said*.

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct ToneMeta {
    /// Printed to stderr at startup so you can confirm which
    /// config/template pairing actually loaded — not read as a
    /// behavioral switch by any bucketing or rendering logic. A place to
    /// name/track which tone a given config+templates pairing represents
    /// (e.g. "neutral-analytical", "casual"), useful once you have more
    /// than one profile directory to choose between.
    #[serde(default)]
    pub profile_name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ActivityConfig {
    pub active_within_days: i64,
    pub maintained_within_days: i64,
    pub label_active: String,
    pub label_maintained: String,
    pub label_dormant: String,
    pub label_unknown: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PopularityConfig {
    pub notable_at_stars: i64,
    pub established_at_stars: i64,
    pub emerging_at_stars: i64,
    pub label_notable: String,
    pub label_established: String,
    pub label_emerging: String,
    pub label_new_or_niche: String,
    pub label_unknown: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ReleaseCadenceConfig {
    pub frequent_at_count: i64,
    pub occasional_at_count: i64,
    pub label_frequent: String,
    pub label_occasional: String,
    pub label_none: String,
    pub label_unknown: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default)]
    pub tone: ToneMeta,
    pub activity: ActivityConfig,
    pub popularity: PopularityConfig,
    pub release_cadence: ReleaseCadenceConfig,
}

impl Default for ToneMeta {
    fn default() -> Self {
        ToneMeta {
            profile_name: "unnamed".to_string(),
        }
    }
}

/// The config `ghbrief` ships with, embedded in the binary at compile
/// time so the tool works with zero setup. `--config` overrides this
/// with a file on disk — see `Config::load`.
const EMBEDDED_DEFAULT_TOML: &str = include_str!("../config/ghbrief.default.toml");

impl Config {
    pub fn embedded_default() -> anyhow::Result<Config> {
        toml::from_str(EMBEDDED_DEFAULT_TOML)
            .map_err(|e| anyhow::anyhow!("ghbrief's own embedded default config failed to parse (this is a bug in ghbrief, not your setup): {e}"))
    }

    /// Loads from `path` if given, otherwise falls back to the embedded
    /// default — printing which one it used, since silently picking a
    /// fallback without saying so is exactly the kind of thing this
    /// project's sibling `ghlinks` avoids.
    pub fn load(path: Option<&std::path::Path>) -> anyhow::Result<Config> {
        match path {
            Some(p) => {
                let text = std::fs::read_to_string(p)
                    .map_err(|e| anyhow::anyhow!("reading config file {}: {e}", p.display()))?;
                toml::from_str(&text)
                    .map_err(|e| anyhow::anyhow!("parsing config file {}: {e}", p.display()))
            }
            None => {
                eprintln!(
                    "ghbrief: no --config given, using the embedded default thresholds/labels"
                );
                Config::embedded_default()
            }
        }
    }
}
