use chrono::{DateTime, Utc};
use serde::Serialize;

/// One recorded app session: spans from tracker start (or resume after suspend)
/// to when it stops. Time is split across active, idle, and locked buckets.
#[derive(Debug, Serialize, Clone)]
pub struct Session {
    pub id: i64,
    pub start_time: String,   // ISO 8601 UTC
    pub end_time: String,     // ISO 8601 UTC; empty string = in-progress
    pub active_secs: i64,
    pub idle_secs: i64,
    pub locked_secs: i64,
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
    /// DB row id of the currently-open app session (–1 while not yet inserted).
    pub current_session_id: i64,
    /// Active seconds accumulated so far in the current session.
    pub current_active_secs: i64,
    /// Idle seconds accumulated so far in the current session.
    pub current_idle_secs: i64,
    /// Locked seconds accumulated so far in the current session.
    pub current_locked_secs: i64,
}
