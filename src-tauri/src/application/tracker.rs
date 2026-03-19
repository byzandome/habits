use std::sync::{Arc, Mutex};

use chrono::{SecondsFormat, Utc};
use tauri::Emitter;

use crate::domain::{
    entities::TrackerState,
    ports::{AppRepository, AppUsageRepository},
};

// ── Constants ─────────────────────────────────────────────────────────────────

/// Real-time gap between polls (seconds) that indicates system suspension.
const SUSPEND_GAP_SECS: i64 = 25;

// ── Tray icon helper ──────────────────────────────────────────────────────────

fn update_tray_icon(app_handle: &tauri::AppHandle, status: &str) {
    let icon = if status == "productive" {
        crate::infrastructure::tray_icon::productive_icon()
    } else {
        crate::infrastructure::tray_icon::idle_icon()
    };
    if let Some(tray) = app_handle.tray_by_id("main-tray") {
        let _ = tray.set_icon(Some(icon));
    }
}

// ── Close the current open app_usage row ─────────────────────────────────────

fn close_current_usage(
    tracker: &mut TrackerState,
    app_usage_repo: &Arc<dyn AppUsageRepository>,
    now: chrono::DateTime<Utc>,
) {
    if let (Some(uid), Some(start)) = (
        tracker.current_app_usage_id.take(),
        tracker.current_usage_start.take(),
    ) {
        let dur = (now - start).num_seconds().max(0);
        let end_str = now.to_rfc3339_opts(SecondsFormat::Secs, true);
        let _ = app_usage_repo.end_usage(&uid, &end_str, dur);
        tracker.current_app_path = None;
    }
}

// ── Background tracking loop ──────────────────────────────────────────────────

pub async fn run_tracker(
    app_repo: Arc<dyn AppRepository>,
    app_usage_repo: Arc<dyn AppUsageRepository>,
    tracker: Arc<Mutex<TrackerState>>,
    app_handle: tauri::AppHandle,
    wake: Arc<tokio::sync::Notify>,
) {
    let mut last_poll_wall = Utc::now();

    loop {
        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {}
            _ = wake.notified() => {}
        }

        let now = Utc::now();
        let actual_gap = (now - last_poll_wall).num_seconds();
        last_poll_wall = now;

        // ── Suspend / sleep detection ─────────────────────────────────────────
        if actual_gap > SUSPEND_GAP_SECS {
            let mut t = tracker.lock().unwrap();
            close_current_usage(&mut t, &app_usage_repo, now);
            t.status = "idle".to_string();
            t.session_start = now;
            update_tray_icon(&app_handle, "idle");
            let _ = app_handle.emit("tracker-status-changed", "idle");
            continue;
        }

        // ── Determine current status ──────────────────────────────────────────
        let idle_secs = crate::infrastructure::idle::get_idle_seconds();
        let is_locked = crate::infrastructure::idle::is_session_locked();

        let threshold = tracker.lock().unwrap().idle_threshold_secs;
        let new_status = if is_locked {
            "locked"
        } else if idle_secs >= threshold {
            "idle"
        } else {
            "productive"
        };

        let prev_status = tracker.lock().unwrap().status.clone();

        if new_status != prev_status {
            update_tray_icon(&app_handle, new_status);
            let _ = app_handle.emit("tracker-status-changed", new_status);

            if new_status != "productive" {
                // Leaving productive state — close the open app_usage row.
                let mut t = tracker.lock().unwrap();
                close_current_usage(&mut t, &app_usage_repo, now);
            }

            tracker.lock().unwrap().status = new_status.to_string();
        }

        // ── App usage tracking (productive state only) ────────────────────────
        if new_status == "productive" {
            let active = crate::infrastructure::active_app::get_active_app();

            let current_path = tracker.lock().unwrap().current_app_path.clone();
            let app_changed = current_path.as_deref() != Some(active.exe_path.as_str());

            if app_changed {
                {
                    let mut t = tracker.lock().unwrap();
                    close_current_usage(&mut t, &app_usage_repo, now);
                }

                if let Ok(app) = app_repo.upsert_app(&active.name, &active.exe_path) {
                    let start_str = now.to_rfc3339_opts(SecondsFormat::Secs, true);
                    if let Ok(uid) = app_usage_repo.begin_usage(&app.id, &start_str) {
                        let mut t = tracker.lock().unwrap();
                        t.current_app_usage_id = Some(uid);
                        t.current_app_path = Some(active.exe_path);
                        t.current_usage_start = Some(now);
                    }
                }
            }
        }
    }
}