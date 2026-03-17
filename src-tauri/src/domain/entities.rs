use chrono::{DateTime, Utc};
use serde::Serialize;

/// The single session for a given local date.
#[derive(Debug, Serialize, Clone)]
pub struct Session {
    pub id: String,
    pub date: String,         // "YYYY-MM-DD" local date
    pub start_time: String,   // ISO 8601 UTC — first app boot of the day
    pub end_time: String,     // ISO 8601 UTC; empty string = in-progress
    pub active_secs: i64,
    pub idle_secs: i64,
    pub locked_secs: i64,
    pub unknown_secs: i64,
}

/// Per-day aggregated summary used for the history view.
#[derive(Debug, Serialize, Clone)]
pub struct DailySummary {
    pub date: String,            // "YYYY-MM-DD" local date
    pub productive_secs: i64,
    pub idle_secs: i64,
    pub locked_secs: i64,
}

/// Per-application usage stat for a given day.
#[derive(Debug, Serialize, Clone)]
pub struct AppUsageStat {
    pub app_name: String,
    pub duration_secs: i64,
    pub exe_path: String,
    pub pct_of_day: f64,
}

/// One interval row: cumulative time for a single status type within a session.
#[derive(Debug, Serialize, Clone)]
pub struct Interval {
    pub id: String,
    pub session_id: String,
    pub app_usage_id: Option<String>,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration_secs: i64,
    #[serde(rename = "type")]
    pub interval_type: String,  // "active" | "idle" | "locked" | "unknown"
}

/// In-memory tracker state — shared between the background tracking loop and
/// the Tauri command handlers via `Arc<Mutex<TrackerState>>`.
pub struct TrackerState {
    /// Current user-activity status: "productive" | "idle" | "locked".
    pub status: String,
    /// UTC timestamp when the current status began (used for live duration display).
    pub session_start: DateTime<Utc>,
    /// Seconds of inactivity that triggers the productive → idle transition.
    pub idle_threshold_secs: u64,
    /// UUID of the currently-open app session (empty string while not yet inserted).
    pub current_session_id: String,
    /// Active seconds accumulated so far in the current session.
    pub current_active_secs: i64,
    /// Idle seconds accumulated so far in the current session.
    pub current_idle_secs: i64,
    /// Locked seconds accumulated so far in the current session.
    pub current_locked_secs: i64,
}
