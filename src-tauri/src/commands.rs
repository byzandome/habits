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
    let duration = (Utc::now() - t.session_start).num_seconds().max(0);
    Ok(CurrentStatus {
        status: t.status.clone(),
        session_duration_secs: duration,
    })
}

/// Returns today's cumulative productive and idle seconds (including in-progress session).
#[tauri::command]
pub fn get_today_stats(state: State<'_, AppState>) -> Result<TodayStats, String> {
    let local_today = Local::now().format("%Y-%m-%d").to_string();

    // Snapshot tracker before releasing its lock
    let (status, session_start) = {
        let t = state.tracker.lock().unwrap();
        (t.status.clone(), t.session_start)
    };

    let (mut prod, mut idle) = {
        let conn = state.db.lock().unwrap();
        db::get_today_stats(&conn, &local_today).map_err(|e| e.to_string())?
    };

    // Add the in-progress session if it started today (local)
    let now = Utc::now();
    let session_local_date = session_start
        .with_timezone(&Local)
        .format("%Y-%m-%d")
        .to_string();
    if session_local_date == local_today {
        let extra = (now - session_start).num_seconds().max(0);
        if status == "productive" {
            prod += extra;
        } else {
            idle += extra;
        }
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
    let now = Utc::now();
    let local_today = Local::now().format("%Y-%m-%d").to_string();

    // Snapshot tracker
    let (status, session_start) = {
        let t = state.tracker.lock().unwrap();
        (t.status.clone(), t.session_start)
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
            let duration = (now - session_start).num_seconds().max(0);
            sessions.push(db::Session {
                id: -1,
                start_time: session_start.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                end_time: String::new(), // empty = in-progress
                session_type: status,
                duration_secs: duration,
            });
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
/// or an empty string when the icon cannot be extracted.
#[tauri::command]
pub fn get_app_icon(app_name: String) -> String {
    crate::app_icon::get_icon_base64(&app_name)
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
