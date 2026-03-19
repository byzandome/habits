use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

// ── Persisted domain types ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct App {
    pub id: String,
    pub name: String,
    pub path: String,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Domain {
    pub id: String,
    pub url: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DomainHistory {
    pub id: String,
    pub domain_id: String,
    pub url: String,
    pub start_at: String,
    pub end_at: Option<String>,
    pub duration_secs: Option<i64>,
}

// ── Aggregated view types ───────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone, QueryableByName)]
pub struct AppUsageStat {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub app_id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub app_name: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub duration_secs: i64,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub start_at: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub end_at: String,
}

// ── In-memory tracker state ─────────────────────────────────────────────────────────────

pub struct TrackerState {
    /// "productive" | "idle" | "locked"
    pub status: String,
    pub session_start: DateTime<Utc>,
    pub idle_threshold_secs: u64,
    /// ID of the currently-open `app_usages` row (None while idle/locked).
    pub current_app_usage_id: Option<String>,
    /// Exe path of the tracked app — used to detect foreground app changes.
    pub current_app_path: Option<String>,
    /// When the current app_usage row was opened.
    pub current_usage_start: Option<DateTime<Utc>>,
}
