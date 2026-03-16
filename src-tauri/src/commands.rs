use chrono::{Local, Utc};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{db, AppState};

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct CurrentStatus {
    pub status: String,
    pub session_duration_secs: i64,
}

#[derive(Serialize)]
pub struct TodayStats {
    pub productive_secs: i64,
    pub idle_secs: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub idle_threshold_mins: u64,
    pub autostart: bool,
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// Returns the current tracking status and how many seconds the current session has lasted.
#[tauri::command]
pub fn get_current_status(state: State<'_, AppState>) -> Result<CurrentStatus, String> {
    let t = state.tracker.lock().unwrap();
    // session_duration_secs shows the total elapsed time of the current app
    // session (active + idle combined) so the UI can display a running clock.
    let duration = (Utc::now() - t.session_start).num_seconds().max(0)
        + t.current_active_secs
        + t.current_idle_secs;
    Ok(CurrentStatus {
        status: t.status.clone(),
        session_duration_secs: duration,
    })
}

/// Returns today's cumulative productive and idle seconds (including in-progress session).
#[tauri::command]
pub fn get_today_stats(state: State<'_, AppState>) -> Result<TodayStats, String> {
    let local_today = Local::now().format("%Y-%m-%d").to_string();

    // Snapshot in-progress counters before releasing the lock.
    let (cur_active, cur_idle, session_start) = {
        let t = state.tracker.lock().unwrap();
        (t.current_active_secs, t.current_idle_secs, t.session_start)
    };

    let (mut prod, mut idle) = {
        let conn = state.db.lock().unwrap();
        db::get_today_stats(&conn, &local_today).map_err(|e| e.to_string())?
    };

    // Add the in-progress session's counters if it started today (local).
    let session_local_date = session_start
        .with_timezone(&Local)
        .format("%Y-%m-%d")
        .to_string();
    if session_local_date == local_today {
        prod += cur_active;
        idle += cur_idle;
    }

    Ok(TodayStats {
        productive_secs: prod,
        idle_secs: idle,
    })
}

/// Returns all sessions (completed + in-progress) for a local date "YYYY-MM-DD".
#[tauri::command]
pub fn get_sessions_for_date(
    state: State<'_, AppState>,
    date: String,
) -> Result<Vec<db::Session>, String> {
    let local_today = Local::now().format("%Y-%m-%d").to_string();

    // Snapshot tracker
    let (cur_active, cur_idle, session_start) = {
        let t = state.tracker.lock().unwrap();
        (t.current_active_secs, t.current_idle_secs, t.session_start)
    };

    let mut sessions = {
        let conn = state.db.lock().unwrap();
        db::get_sessions_for_date(&conn, &date).map_err(|e| e.to_string())?
    };

    // Append in-progress session when querying today
    if date == local_today {
        let session_local_date = session_start
            .with_timezone(&Local)
            .format("%Y-%m-%d")
            .to_string();
        if session_local_date == local_today {
            // Remove the placeholder row (if any) inserted by begin_session and
            // replace it with a live view that has up-to-date counters.
            sessions.retain(|s| s.end_time != "");
            sessions.push(db::Session {
                id: -1,
                start_time: session_start
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                end_time: String::new(), // empty = in-progress
                active_secs: cur_active,
                idle_secs: cur_idle,
            });
            sessions.sort_by(|a, b| b.start_time.cmp(&a.start_time)); // newest first
        }
    }

    Ok(sessions)
}

/// Returns per-day summaries for the last `days` days (default 7), newest first.
#[tauri::command]
pub fn get_history(
    state: State<'_, AppState>,
    days: Option<u32>,
) -> Result<Vec<db::DailySummary>, String> {
    let conn = state.db.lock().unwrap();
    db::get_history(&conn, days.unwrap_or(7)).map_err(|e| e.to_string())
}

/// Returns current app settings.
#[tauri::command]
pub fn get_settings(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<Settings, String> {
    let conn = state.db.lock().unwrap();

    let threshold_mins = db::get_setting(&conn, "idle_threshold_mins")
        .ok()
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
) -> Result<Vec<db::AppUsageStat>, String> {
    let conn = state.db.lock().unwrap();
    db::get_app_usage_for_date(&conn, &date).map_err(|e| e.to_string())
}

/// Returns a `"data:image/png;base64,…"` string for the given process stem,
/// or an empty string when the exe path is unknown (frontend shows placeholder).
#[tauri::command]
pub fn get_app_icon(app_name: String, state: State<'_, AppState>) -> String {
    let conn = state.db.lock().unwrap();
    let exe_path = crate::db::get_exe_path_for_app(&conn, &app_name)
        .ok()
        .flatten()
        .unwrap_or_default();
    crate::app_icon::get_icon_base64_from_path(&exe_path)
}

/// Clears all cached exe paths stored in the DB so icons are re-resolved on
/// the next request.  The frontend module-level icon cache is not accessible
/// from Rust, so the frontend is responsible for clearing it after this call.
#[tauri::command]
pub fn clear_icon_cache(state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    crate::db::clear_exe_path_cache(&conn).map_err(|e| e.to_string())?;
    crate::app_icon::clear_in_memory_cache();
    Ok(())
}

/// Deletes all recorded sessions and app-usage data, preserving settings.
/// The tracker keeps running normally; a fresh session begins on the next poll.
#[tauri::command]
pub fn clear_all_data(state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    crate::db::clear_all_data(&conn).map_err(|e| e.to_string())
}

/// Persists settings and applies them immediately.
#[tauri::command]
pub fn set_settings(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    idle_threshold_mins: u64,
    autostart: bool,
) -> Result<(), String> {
    // Update live threshold
    {
        let mut t = state.tracker.lock().unwrap();
        t.idle_threshold_secs = idle_threshold_mins * 60;
    }

    // Persist to DB
    {
        let conn = state.db.lock().unwrap();
        db::set_setting(&conn, "idle_threshold_mins", &idle_threshold_mins.to_string())
            .map_err(|e| e.to_string())?;
        db::set_setting(&conn, "autostart", &autostart.to_string())
            .map_err(|e| e.to_string())?;
    }

    // Toggle OS autostart
    use tauri_plugin_autostart::ManagerExt;
    if autostart {
        app.autolaunch().enable().map_err(|e| e.to_string())?;
    } else {
        app.autolaunch().disable().map_err(|e| e.to_string())?;
    }

    Ok(())
}
