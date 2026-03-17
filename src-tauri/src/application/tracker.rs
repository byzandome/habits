use std::sync::{Arc, Mutex};

use chrono::{Local, Utc};
use tauri::Emitter;

use crate::domain::{
    entities::TrackerState,
    ports::{AppUsageRepository, SessionRepository},
};

// ── Constants ─────────────────────────────────────────────────────────────────

/// Number of 5-second polls before force-writing accumulated counters to DB.
/// 60 × 5 s = 5 minutes — limits data loss if the process is killed.
const CHECKPOINT_POLLS: u32 = 60;

/// Real-time gap between polls (seconds) that indicates system suspension.
/// 25 s = 5× the poll interval, so normal jitter never triggers this.
const SUSPEND_GAP_SECS: i64 = 25;

/// Maximum seconds credited to a single poll cycle to guard against
/// false accumulation when the timer wakes late.
const MAX_ELAPSED_SECS: i64 = 10;

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

// ── Background tracking loop ──────────────────────────────────────────────────

/// Polls every 5 seconds (or immediately on lock/unlock via `wake`).
///
/// **Session model (Teams-style):**
/// One app session spans from the moment the tracker starts (or the system
/// resumes after suspend) until the app is quit or the system suspends again.
/// Within a session every poll slice is added to `active_secs` when the user
/// was interacting with the PC, `idle_secs` when idle time exceeds the
/// threshold, or `locked_secs` when the screen is locked. No new session row
/// is created merely because the user went idle or locked.
///
/// **Suspend / sleep / hibernate handling:**
/// When the real wall-clock gap between two polls exceeds `SUSPEND_GAP_SECS`
/// the system was suspended. The pre-suspend session is closed and a fresh one
/// opened, so the suspension period is never silently attributed to any bucket.
pub async fn run_tracker(
    session_repo: Arc<dyn SessionRepository>,
    app_usage_repo: Arc<dyn AppUsageRepository>,
    tracker: Arc<Mutex<TrackerState>>,
    app_handle: tauri::AppHandle,
    wake: Arc<tokio::sync::Notify>,
) {
    let mut poll_count: u32 = 0;
    let mut last_poll_wall = Utc::now();
    let mut last_app_tick = Utc::now();

    loop {
        // Wake on the 5-second timer OR immediately on lock/unlock.
        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {}
            _ = wake.notified() => {}
        }

        let now = Utc::now();
        let actual_gap_secs = (now - last_poll_wall).num_seconds();
        last_poll_wall = now;

        // ── Suspend / sleep / hibernate detection ────────────────────────────
        if actual_gap_secs > SUSPEND_GAP_SECS {
            // The system was suspended. Just reset the in-memory counters so
            // time during suspension is not attributed to any bucket.  The same
            // day's session (one per date) is reused; we do NOT close it.
            {
                let mut t = tracker.lock().unwrap();
                t.session_start = now;
                t.status = "idle".to_string();
                poll_count = 0;
            }

            update_tray_icon(&app_handle, "idle");
            last_app_tick = now;
            continue;
        }

        poll_count += 1;

        let idle_secs_system = crate::infrastructure::idle::get_idle_seconds();
        let now = Utc::now();
        let elapsed = actual_gap_secs.max(0).min(MAX_ELAPSED_SECS);

        // ── Accumulate time & detect status transitions ───────────────────────
        let is_locked = crate::infrastructure::idle::is_session_locked();
        let new_status: Option<String> = {
            let mut t = tracker.lock().unwrap();
            let prev_status = t.status.clone();
            let threshold = t.idle_threshold_secs;

            if is_locked {
                t.current_locked_secs += elapsed;
                if t.status != "locked" {
                    t.session_start = now;
                    t.status = "locked".to_string();
                    poll_count = 0;
                }
            } else if t.status == "productive" && idle_secs_system >= threshold {
                // productive → idle: retroactively transfer backlog to idle bucket
                let idle_secs_i64 = idle_secs_system as i64;
                let active_slice = (elapsed - idle_secs_i64).max(0);
                let idle_slice = elapsed - active_slice;
                let backlog = (idle_secs_i64 - elapsed)
                    .max(0)
                    .min(t.current_active_secs);
                t.current_active_secs -= backlog;
                t.current_idle_secs += backlog;
                t.current_active_secs += active_slice;
                t.current_idle_secs += idle_slice;
                t.session_start = now - chrono::Duration::seconds(idle_secs_i64);
                t.status = "idle".to_string();
                poll_count = 0;
            } else if t.status != "productive" && idle_secs_system < threshold {
                // idle or locked → productive
                let idle_secs_i64 = idle_secs_system as i64;
                let active_part = idle_secs_i64.min(elapsed);
                let idle_part = elapsed - active_part;
                match t.status.as_str() {
                    "idle" => {
                        t.current_idle_secs += idle_part;
                        t.current_active_secs += active_part;
                    }
                    "locked" => {
                        t.current_locked_secs += idle_part;
                        t.current_active_secs += active_part;
                    }
                    _ => {}
                }
                t.session_start = now - chrono::Duration::seconds(idle_secs_i64);
                t.status = "productive".to_string();
                poll_count = 0;
            } else if t.status == "productive" {
                t.current_active_secs += elapsed;
            } else if t.status == "idle" {
                t.current_idle_secs += elapsed;
            } else {
                t.current_locked_secs += elapsed;
            }

            // ── Periodic DB checkpoint ────────────────────────────────────────
            if poll_count >= CHECKPOINT_POLLS {
                let _ = session_repo.update_session_time(
                    &t.current_session_id,
                    t.current_active_secs,
                    t.current_idle_secs,
                    t.current_locked_secs,
                );
                poll_count = 0;
            }

            if t.status != prev_status {
                Some(t.status.clone())
            } else {
                None
            }
        };

        if let Some(status) = new_status {
            update_tray_icon(&app_handle, &status);
            let _ = app_handle.emit("tracker-status-changed", &status);
        }

        // ── App usage tracking ────────────────────────────────────────────────
        let current_status = {
            let t = tracker.lock().unwrap();
            t.status.clone()
        };
        if current_status == "productive" {
            let app_elapsed = (now - last_app_tick).num_seconds().max(0).min(MAX_ELAPSED_SECS);
            if app_elapsed > 0 {
                let active_app = crate::infrastructure::active_app::get_active_app();
                let local_date = now.with_timezone(&Local).format("%Y-%m-%d").to_string();
                let _ = app_usage_repo.upsert_app_usage(
                    &active_app.name,
                    &local_date,
                    app_elapsed,
                    &active_app.exe_path,
                );
            }
        }
        last_app_tick = now;
    }
}
