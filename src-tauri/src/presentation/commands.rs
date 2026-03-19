use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    domain::ports::{AppRepository, AppUsageRepository, DomainHistoryRepository, DomainRepository, SettingsRepository},
    infrastructure::icon,
    AppState,
};

// ── Response / request types ──────────────────────────────────────────────────

#[derive(Serialize)]
pub struct CurrentStatus {
    pub status: String,
    pub elapsed_secs: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub idle_threshold_mins: u64,
    pub autostart: bool,
}

// ── Commands ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_current_status(state: State<'_, AppState>) -> Result<CurrentStatus, String> {
    let t = state.tracker.lock().unwrap();
    let elapsed = (Utc::now() - t.session_start).num_seconds().max(0);
    Ok(CurrentStatus { status: t.status.clone(), elapsed_secs: elapsed })
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>, app: tauri::AppHandle) -> Result<Settings, String> {
    let threshold_mins = state
        .db
        .get_setting("idle_threshold_mins")?
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(5);

    use tauri_plugin_autostart::ManagerExt;
    let autostart = app.autolaunch().is_enabled().unwrap_or(false);

    Ok(Settings { idle_threshold_mins: threshold_mins, autostart })
}

#[tauri::command]
pub fn set_settings(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    idle_threshold_mins: u64,
    autostart: bool,
    theme: String,
    lang: String,
) -> Result<(), String> {
    {
        let mut t = state.tracker.lock().unwrap();
        t.idle_threshold_secs = idle_threshold_mins * 60;
    }
    state.db.set_setting("idle_threshold_mins", &idle_threshold_mins.to_string())?;
    state.db.set_setting("autostart", &autostart.to_string())?;
    state.db.set_setting("theme", &theme)?;
    state.db.set_setting("lang", &lang)?;
    
    use tauri_plugin_autostart::ManagerExt;
    if autostart {
        app.autolaunch().enable().map_err(|e| e.to_string())?;
    } else {
        app.autolaunch().disable().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_apps(state: State<'_, AppState>) -> Result<Vec<crate::domain::entities::App>, String> {
    state.db.list_apps()
}

#[tauri::command]
pub fn get_app_usages(
    state: State<'_, AppState>,
    date: Option<String>,
) -> Result<Vec<crate::domain::entities::AppUsage>, String> {
    state.db.list_usages(date.as_deref())
}

#[tauri::command]
pub fn get_domains(state: State<'_, AppState>) -> Result<Vec<crate::domain::entities::Domain>, String> {
    state.db.list_domains()
}

#[tauri::command]
pub fn get_domain_history(
    state: State<'_, AppState>,
    date: Option<String>,
) -> Result<Vec<crate::domain::entities::DomainHistory>, String> {
    state.db.list_history(date.as_deref())
}

#[tauri::command]
pub fn get_app_icon(
    state: State<'_, AppState>,
    app_name: String,
) -> Result<String, String> {
    let app = state.db.find_app_by_name(&app_name)?;

    let exe_path = match &app {
        Some(a) => a.path.as_str(),
        None => return Ok(String::new()),
    };

    let (data_uri, color) = icon::ensure_icon_cached(&state.icons_dir, &app_name, exe_path);

    // Persist the dominant colour if we extracted one and the app doesn't have one yet.
    if let (Some(ref c), Some(ref a)) = (&color, &app) {
        if a.color.is_none() {
            let _ = state.db.update_app_color(&a.id, Some(c));
        }
    }

    Ok(data_uri.unwrap_or_default())
}

#[tauri::command]
pub fn clear_icon_cache(state: State<'_, AppState>) -> Result<(), String> {
    icon::clear_cache(&state.icons_dir)?;
    state.db.reset_all_colors()
}