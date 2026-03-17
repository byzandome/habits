use chrono::{DateTime, Utc};

use super::entities::{AppUsageStat, DailySummary, Session};

// ── Session persistence ───────────────────────────────────────────────────────

pub trait SessionRepository: Send + Sync {
    fn begin_session(&self, start: &DateTime<Utc>) -> Result<i64, String>;
    fn update_session_time(
        &self,
        id: i64,
        active_secs: i64,
        idle_secs: i64,
        locked_secs: i64,
    ) -> Result<(), String>;
    fn end_session(&self, id: i64, end: &DateTime<Utc>) -> Result<(), String>;
    fn get_sessions_for_date(&self, date: &str) -> Result<Vec<Session>, String>;
    fn get_today_stats(&self, date: &str) -> Result<(i64, i64, i64), String>;
    fn get_history(&self, days: u32) -> Result<Vec<DailySummary>, String>;
    fn clear_all_data(&self) -> Result<(), String>;
}

// ── App usage persistence ─────────────────────────────────────────────────────

pub trait AppUsageRepository: Send + Sync {
    fn upsert_app_usage(
        &self,
        app_name: &str,
        date: &str,
        duration_secs: i64,
        exe_path: &str,
    ) -> Result<(), String>;
    fn get_app_usage_for_date(&self, date: &str) -> Result<Vec<AppUsageStat>, String>;
    fn get_exe_path_for_app(&self, app_name: &str) -> Result<Option<String>, String>;
    fn clear_exe_path_cache(&self) -> Result<(), String>;
}

// ── Settings persistence ──────────────────────────────────────────────────────

pub trait SettingsRepository: Send + Sync {
    fn get_setting(&self, key: &str) -> Result<Option<String>, String>;
    fn set_setting(&self, key: &str, value: &str) -> Result<(), String>;
}
