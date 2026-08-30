//! Pure functions deriving a small, honest set of labeled facts from a
//! `ghlinks` record. No I/O, no template rendering — these take already-
//! parsed data plus a `Config` and return a bucket. Kept separate from
//! `render.rs` specifically so they're unit-testable in isolation, the
//! same separation-of-concerns `ghlinks` itself uses for `retry.rs`.
//!
//! The recurring pattern here — a distinct `Unknown` variant on every
//! enum, never collapsed into the "worst" or "lowest" real bucket — is
//! deliberate: a missing `pushed_at` is not evidence of dormancy, and
//! this code should never manufacture that inference just because a
//! definite answer is more satisfying to read than an honest one.

use crate::config::{ActivityConfig, PopularityConfig, ReleaseCadenceConfig};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityBucket {
    Active,
    Maintained,
    Dormant,
    Unknown,
}

impl ActivityBucket {
    pub fn label<'a>(&self, cfg: &'a ActivityConfig) -> &'a str {
        match self {
            ActivityBucket::Active => &cfg.label_active,
            ActivityBucket::Maintained => &cfg.label_maintained,
            ActivityBucket::Dormant => &cfg.label_dormant,
            ActivityBucket::Unknown => &cfg.label_unknown,
        }
    }
}

/// `now` is passed in (rather than read via `Utc::now()` internally) so
/// this stays a pure, deterministic, easily-unit-tested function — the
/// same reason `ghlinks`'s own age/recency logic threads a timestamp
/// through rather than calling a clock mid-computation.
pub fn activity_bucket(
    pushed_at: Option<&str>,
    now: DateTime<Utc>,
    cfg: &ActivityConfig,
) -> ActivityBucket {
    let Some(raw) = pushed_at else {
        return ActivityBucket::Unknown;
    };
    let Ok(parsed) = DateTime::parse_from_rfc3339(raw) else {
        return ActivityBucket::Unknown;
    };
    let days_since = (now - parsed.with_timezone(&Utc)).num_days();
    if days_since < 0 {
        // A push date in ghbrief's own future is a clock-skew/bad-data
        // situation, not a confident "very active" signal — treat it the
        // same as not knowing, rather than asserting something odd.
        return ActivityBucket::Unknown;
    }
    if days_since <= cfg.active_within_days {
        ActivityBucket::Active
    } else if days_since <= cfg.maintained_within_days {
        ActivityBucket::Maintained
    } else {
        ActivityBucket::Dormant
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopularityBucket {
    Notable,
    Established,
    Emerging,
    NewOrNiche,
    Unknown,
}

impl PopularityBucket {
    pub fn label<'a>(&self, cfg: &'a PopularityConfig) -> &'a str {
        match self {
            PopularityBucket::Notable => &cfg.label_notable,
            PopularityBucket::Established => &cfg.label_established,
            PopularityBucket::Emerging => &cfg.label_emerging,
            PopularityBucket::NewOrNiche => &cfg.label_new_or_niche,
            PopularityBucket::Unknown => &cfg.label_unknown,
        }
    }
}

pub fn popularity_bucket(stars: Option<i64>, cfg: &PopularityConfig) -> PopularityBucket {
    let Some(stars) = stars else {
        return PopularityBucket::Unknown;
    };
    if stars >= cfg.notable_at_stars {
        PopularityBucket::Notable
    } else if stars >= cfg.established_at_stars {
        PopularityBucket::Established
    } else if stars >= cfg.emerging_at_stars {
        PopularityBucket::Emerging
    } else {
        PopularityBucket::NewOrNiche
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseCadenceBucket {
    Frequent,
    Occasional,
    NoRecentReleases,
    Unknown,
}

impl ReleaseCadenceBucket {
    pub fn label<'a>(&self, cfg: &'a ReleaseCadenceConfig) -> &'a str {
        match self {
            ReleaseCadenceBucket::Frequent => &cfg.label_frequent,
            ReleaseCadenceBucket::Occasional => &cfg.label_occasional,
            ReleaseCadenceBucket::NoRecentReleases => &cfg.label_none,
            ReleaseCadenceBucket::Unknown => &cfg.label_unknown,
        }
    }
}

/// Deliberately independent of `activity_bucket`: a repo can be pushed
/// to daily with zero formal releases in the last year, and conflating
/// "no releases" with "inactive" would misrepresent exactly the kind of
/// project this distinction exists to protect against.
pub fn release_cadence_bucket(
    releases_last_12_months: Option<i64>,
    cfg: &ReleaseCadenceConfig,
) -> ReleaseCadenceBucket {
    let Some(count) = releases_last_12_months else {
        return ReleaseCadenceBucket::Unknown;
    };
    if count >= cfg.frequent_at_count {
        ReleaseCadenceBucket::Frequent
    } else if count >= cfg.occasional_at_count {
        ReleaseCadenceBucket::Occasional
    } else {
        ReleaseCadenceBucket::NoRecentReleases
    }
}

/// Top N languages by byte count, descending, ties broken by name for
/// determinism (`BTreeMap` iteration order is by key, not by value, so
/// this can't just take the map's natural order).
pub fn top_languages(
    languages_bytes: &std::collections::BTreeMap<String, i64>,
    n: usize,
) -> Vec<String> {
    let mut entries: Vec<(&String, &i64)> = languages_bytes.iter().collect();
    entries.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    entries
        .into_iter()
        .take(n)
        .map(|(name, _)| name.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn cfg() -> Config {
        Config::embedded_default().expect("embedded default config must parse")
    }

    fn days_ago(now: DateTime<Utc>, days: i64) -> String {
        (now - chrono::Duration::days(days)).to_rfc3339()
    }

    #[test]
    fn activity_bucket_none_is_unknown_not_dormant() {
        let now = Utc::now();
        assert_eq!(
            activity_bucket(None, now, &cfg().activity),
            ActivityBucket::Unknown
        );
    }

    #[test]
    fn activity_bucket_unparseable_date_is_unknown() {
        let now = Utc::now();
        assert_eq!(
            activity_bucket(Some("not a date"), now, &cfg().activity),
            ActivityBucket::Unknown
        );
    }

    #[test]
    fn activity_bucket_future_date_is_unknown_not_active() {
        let now = Utc::now();
        let future = (now + chrono::Duration::days(5)).to_rfc3339();
        assert_eq!(
            activity_bucket(Some(&future), now, &cfg().activity),
            ActivityBucket::Unknown
        );
    }

    #[test]
    fn activity_bucket_boundaries() {
        let now = Utc::now();
        let c = cfg();
        assert_eq!(
            activity_bucket(Some(&days_ago(now, 0)), now, &c.activity),
            ActivityBucket::Active
        );
        assert_eq!(
            activity_bucket(
                Some(&days_ago(now, c.activity.active_within_days)),
                now,
                &c.activity
            ),
            ActivityBucket::Active,
            "the threshold day itself must count as still-active"
        );
        assert_eq!(
            activity_bucket(
                Some(&days_ago(now, c.activity.active_within_days + 1)),
                now,
                &c.activity
            ),
            ActivityBucket::Maintained
        );
        assert_eq!(
            activity_bucket(
                Some(&days_ago(now, c.activity.maintained_within_days + 1)),
                now,
                &c.activity
            ),
            ActivityBucket::Dormant
        );
    }

    #[test]
    fn popularity_bucket_none_is_unknown() {
        assert_eq!(
            popularity_bucket(None, &cfg().popularity),
            PopularityBucket::Unknown
        );
    }

    #[test]
    fn popularity_bucket_boundaries() {
        let c = cfg().popularity;
        assert_eq!(
            popularity_bucket(Some(c.notable_at_stars), &c),
            PopularityBucket::Notable
        );
        assert_eq!(
            popularity_bucket(Some(c.notable_at_stars - 1), &c),
            PopularityBucket::Established
        );
        assert_eq!(
            popularity_bucket(Some(c.established_at_stars), &c),
            PopularityBucket::Established
        );
        assert_eq!(
            popularity_bucket(Some(c.emerging_at_stars), &c),
            PopularityBucket::Emerging
        );
        assert_eq!(popularity_bucket(Some(0), &c), PopularityBucket::NewOrNiche);
        assert_eq!(
            popularity_bucket(Some(-1), &c),
            PopularityBucket::NewOrNiche,
            "should never panic on unexpected negative input"
        );
    }

    #[test]
    fn release_cadence_none_is_unknown_not_zero() {
        assert_eq!(
            release_cadence_bucket(None, &cfg().release_cadence),
            ReleaseCadenceBucket::Unknown,
            "a value that was never fetched must not be presented the same as a confirmed zero"
        );
    }

    #[test]
    fn release_cadence_zero_is_no_recent_releases_not_unknown() {
        assert_eq!(
            release_cadence_bucket(Some(0), &cfg().release_cadence),
            ReleaseCadenceBucket::NoRecentReleases
        );
    }

    #[test]
    fn release_cadence_boundaries() {
        let c = cfg().release_cadence;
        assert_eq!(
            release_cadence_bucket(Some(c.frequent_at_count), &c),
            ReleaseCadenceBucket::Frequent
        );
        assert_eq!(
            release_cadence_bucket(Some(c.occasional_at_count), &c),
            ReleaseCadenceBucket::Occasional
        );
        assert_eq!(
            release_cadence_bucket(Some(c.occasional_at_count - 1), &c),
            ReleaseCadenceBucket::NoRecentReleases
        );
    }

    #[test]
    fn top_languages_orders_by_bytes_descending_with_deterministic_tiebreak() {
        let mut map = std::collections::BTreeMap::new();
        map.insert("Rust".to_string(), 500);
        map.insert("Python".to_string(), 1500);
        map.insert("Shell".to_string(), 1500);
        map.insert("HTML".to_string(), 10);
        let top = top_languages(&map, 2);
        // Python and Shell tie at 1500 bytes; alphabetical tiebreak picks
        // Python first, deterministically, rather than depending on
        // whatever order the map happened to iterate in.
        assert_eq!(top, vec!["Python".to_string(), "Shell".to_string()]);
    }

    #[test]
    fn top_languages_handles_fewer_entries_than_n() {
        let mut map = std::collections::BTreeMap::new();
        map.insert("Rust".to_string(), 100);
        assert_eq!(top_languages(&map, 5), vec!["Rust".to_string()]);
    }

    #[test]
    fn top_languages_handles_empty_map() {
        let map = std::collections::BTreeMap::new();
        assert!(top_languages(&map, 3).is_empty());
    }
}
