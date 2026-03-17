use chrono::{Local, Utc};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    domain::ports::{AppUsageRepository, SessionRepository, SettingsRepository},
    AppState,
};

// ── Response / request types ──────────────────────────────────────────────────

#[derive(Serialize)]
pub struct CurrentStatus {
    pub status: String,
    pub session_duration_secs: i64,
}

#[derive(Serialize)]
pub struct TodayStats {
    pub productive_secs: i64,
    pub idle_secs: i64,
    pub locked_secs: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub idle_threshold_mins: u64,
    pub autostart: bool,
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// Returns the current tracking status and elapsed seconds of the current
/// session so the UI can display a live running clock.
#[tauri::command]
pub fn get_current_status(state: State<'_, AppState>) -> Result<CurrentStatus, String> {

    let t = state.tracker.lock().unwrap();
    let duration = (Utc::now() - t.session_start).num_seconds().max(0)
        + t.current_active_secs
        + t.current_idle_secs;
    Ok(CurrentStatus {
        status: t.status.clone(),
        session_duration_secs: duration,
    })
}

/// Returns today's cumulative productive, idle, and locked seconds (including
/// the in-progress session).
#[tauri::command]
pub fn get_today_stats(state: State<'_, AppState>) -> Result<TodayStats, String> {
    let local_today = Local::now().format("%Y-%m-%d").to_string();
    let (cur_active, cur_idle, cur_locked) = {
        let t = state.tracker.lock().unwrap();
        (t.current_active_secs, t.current_idle_secs, t.current_locked_secs)
    };

    let (mut prod, mut idle, mut locked) = state.db.get_today_stats(&local_today)?;
    prod   += cur_active;
    idle   += cur_idle;
    locked += cur_locked;

    Ok(TodayStats {
        productive_secs: prod,
        idle_secs: idle,
        locked_secs: locked,
    })
}

/// Returns the session for a local date "YYYY-MM-DD", including live in-memory
/// counters merged in when the date is today.
#[tauri::command]
pub fn get_session_for_date(
    state: State<'_, AppState>,
    date: String,
) -> Result<Option<crate::domain::entities::Session>, String> {
    use chrono::Local;
    let local_today = Local::now().format("%Y-%m-%d").to_string();

    let mut session = state.db.get_session_for_date(&date)?;

    // Merge live in-memory counters into today's (possibly still-open) session.
    if date == local_today {
        let t = state.tracker.lock().unwrap();
        let entry = session.get_or_insert_with(|| crate::domain::entities::Session {
            id: t.current_session_id.clone(),
            date: date.clone(),
            start_time: t.session_start.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            end_time: String::new(),
            active_secs: 0,
            idle_secs: 0,
            locked_secs: 0,
            unknown_secs: 0,
        });
        entry.active_secs += t.current_active_secs;
        entry.idle_secs   += t.current_idle_secs;
        entry.locked_secs += t.current_locked_secs;
        entry.end_time = String::new(); // still ongoing
    }

    Ok(session)
}

/// Returns per-day summaries for the last `days` days (default 7), newest first.
#[tauri::command]
pub fn get_history(
    state: State<'_, AppState>,
    days: Option<u32>,
) -> Result<Vec<crate::domain::entities::DailySummary>, String> {
    state.db.get_history(days.unwrap_or(7))
}

/// Returns current app settings.
#[tauri::command]
pub fn get_settings(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<Settings, String> {
    let threshold_mins = state
        .db
        .get_setting("idle_threshold_mins")?
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(5);

    use tauri_plugin_autostart::ManagerExt;
    let autostart = app.autolaunch().is_enabled().unwrap_or(false);

    Ok(Settings {
        idle_threshold_mins: threshold_mins,
        autostart,
    })
}

/// Returns per-app productive time for a given local date "YYYY-MM-DD".
#[tauri::command]
pub fn get_app_usage(
    state: State<'_, AppState>,
    date: String,
) -> Result<Vec<crate::domain::entities::AppUsageStat>, String> {
    state.db.get_app_usage_for_date(&date)
}

/// Returns a `"data:image/png;base64,…"` string for the given process stem,
/// or an empty string when the exe path is unknown.
#[tauri::command]
pub fn get_app_icon(app_name: String, state: State<'_, AppState>) -> String {
    let exe_path = state
        .db
        .get_exe_path_for_app(&app_name)
        .ok()
        .flatten()
        .unwrap_or_default();
    crate::infrastructure::app_icon::get_icon_base64_from_path(&exe_path)
}

/// Clears all cached exe paths so icons are re-resolved on the next request.
#[tauri::command]
pub fn clear_icon_cache(state: State<'_, AppState>) -> Result<(), String> {
    state.db.clear_exe_path_cache()?;
    crate::infrastructure::app_icon::clear_in_memory_cache();
    Ok(())
}

/// Deletes all recorded sessions and app-usage data, preserving settings.
#[tauri::command]
pub fn clear_all_data(state: State<'_, AppState>) -> Result<(), String> {
    state.db.clear_all_data()
}

/// Returns all intervals for a given session UUID.
#[tauri::command]
pub fn get_intervals_for_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<crate::domain::entities::Interval>, String> {
    state.db.get_intervals_for_session(&session_id)
}

/// Persists settings and applies them immediately (threshold + OS autostart).
#[tauri::command]
pub fn set_settings(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    idle_threshold_mins: u64,
    autostart: bool,
) -> Result<(), String> {
    {
        let mut t = state.tracker.lock().unwrap();
        t.idle_threshold_secs = idle_threshold_mins * 60;
    }

    state
        .db
        .set_setting("idle_threshold_mins", &idle_threshold_mins.to_string())?;
    state
        .db
        .set_setting("autostart", &autostart.to_string())?;

    use tauri_plugin_autostart::ManagerExt;
    if autostart {
        app.autolaunch().enable().map_err(|e| e.to_string())?;
    } else {
        app.autolaunch().disable().map_err(|e| e.to_string())?;
    }

    Ok(())
}
