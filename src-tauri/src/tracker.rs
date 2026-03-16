use std::sync::{Arc, Mutex};

use chrono::{DateTime, Local, Utc};
use rusqlite::Connection;
use tauri::Emitter;

// ── Shared tracker state ──────────────────────────────────────────────────────

pub struct TrackerShared {
    /// "productive" | "idle" | "locked" — current user-activity state.
    pub status: String,
    /// UTC timestamp when the current productive/idle/locked state began (used
    /// for the live session-duration display in the UI).
    pub session_start: DateTime<Utc>,
    /// Seconds of inactivity that triggers the productive → idle transition.
    pub idle_threshold_secs: u64,
    /// DB row id of the currently open app session (–1 while not yet inserted).
    pub current_session_id: i64,
    /// Active seconds accumulated so far in the current app session.
    pub current_active_secs: i64,
    /// Idle seconds (threshold-based inactivity) accumulated so far.
    pub current_idle_secs: i64,
    /// Locked seconds (screen-lock) accumulated so far in the current session.
    pub current_locked_secs: i64,
}

// ── Background task ───────────────────────────────────────────────────────────

/// How many 5-second polls before force-writing the accumulated counters to DB.
/// 60 × 5 s = 5 minutes — limits data loss if the process is killed.
const CHECKPOINT_POLLS: u32 = 60;

/// Real-time gap between polls (seconds) that indicates the system was
/// suspended.  25 s = 5× the poll interval, so normal jitter never fires this.
const SUSPEND_GAP_SECS: i64 = 25;

/// Maximum seconds credited to a single poll cycle.  Guards against false
/// accumulation when the timer wakes up late for any reason other than suspend.
const MAX_ELAPSED_SECS: i64 = 10;

/// Polls every 5 seconds (or immediately on lock/unlock via `wake`).
///
/// **Session model (Teams-style):**  
/// One *app session* spans from the moment the tracker starts (or the system
/// resumes after suspend) until the app is quit or the system suspends again.
/// Within a session every poll slice is added to `active_secs` when the user
/// was interacting with the PC, `idle_secs` when idle time exceeds the
/// threshold, or `locked_secs` when the screen is locked.  No new session row
/// is created merely because the user went idle or locked.
///
/// **Suspend / sleep / hibernate handling:**  
/// When the real wall-clock gap between two polls is larger than
/// `SUSPEND_GAP_SECS` we know the system was suspended.  We close the
/// pre-suspend session and immediately open a fresh one, so the suspension
/// period is never silently attributed to any bucket.
fn update_tray_icon(app_handle: &tauri::AppHandle, status: &str) {
    let icon = if status == "productive" {
        crate::tray_icon::productive_icon()
    } else {
        crate::tray_icon::idle_icon()
    };
    if let Some(tray) = app_handle.tray_by_id("main-tray") {
        let _ = tray.set_icon(Some(icon));
    }
}

pub async fn run_tracker(
    db: Arc<Mutex<Connection>>,
    tracker: Arc<Mutex<TrackerShared>>,
    app_handle: tauri::AppHandle,
    wake: std::sync::Arc<tokio::sync::Notify>,
) {
    let mut poll_count: u32 = 0;
    let mut last_poll_wall = Utc::now();
    let mut last_app_tick = Utc::now();

    loop {
        // Wake either on the 5-second timer OR immediately when the WTS
        // thread fires a lock/unlock event.
        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {}
            _ = wake.notified() => {}
        }

        let now = Utc::now();
        let actual_gap_secs = (now - last_poll_wall).num_seconds();
        last_poll_wall = now;

        // ── Suspend / sleep / hibernate detection ────────────────────────────
        if actual_gap_secs > SUSPEND_GAP_SECS {
            // 1. Flush and close the pre-suspend session.
            // 2. Open a new session starting now (user will resume as idle until
            //    the next productive transition).
            {
                let mut t = tracker.lock().unwrap();
                if let Ok(conn) = db.lock() {
                    let _ = crate::db::update_session_time(
                        &conn,
                        t.current_session_id,
                        t.current_active_secs,
                        t.current_idle_secs,
                        t.current_locked_secs,
                    );
                    let _ = crate::db::end_session(&conn, t.current_session_id, &now);

                    if let Ok(new_id) = crate::db::begin_session(&conn, &now) {
                        t.current_session_id = new_id;
                    }
                }
                // Resume in idle state; first real keystroke/mouse-move will
                // trigger the → productive transition on the next poll.
                t.session_start = now;
                t.status = "idle".to_string();
                t.current_active_secs = 0;
                t.current_idle_secs = 0;
                t.current_locked_secs = 0;
                poll_count = 0;
            }

            update_tray_icon(&app_handle, "idle");
            last_app_tick = now;
            continue; // skip normal idle-detection this poll
        }

        poll_count += 1;

        let idle_secs_system = crate::idle::get_idle_seconds();
        let now = Utc::now();

        // Elapsed since last poll, capped to guard against stale timers.
        let elapsed = actual_gap_secs.max(0).min(MAX_ELAPSED_SECS);

        // ── Accumulate time & detect productive ↔ idle ↔ locked transitions ───
        let is_locked = crate::idle::is_session_locked();
        let new_status: Option<String> = {
            let mut t = tracker.lock().unwrap();
            let prev_status = t.status.clone();
            let threshold = t.idle_threshold_secs;

            if is_locked {
                // Screen is locked — all elapsed time credits to locked_secs.
                t.current_locked_secs += elapsed;
                if t.status != "locked" {
                    t.session_start = now;
                    t.status = "locked".to_string();
                    poll_count = 0;
                }
            } else if t.status == "productive" && idle_secs_system >= threshold {
                // productive → idle.
                //
                // The LAST input happened `idle_secs_system` seconds ago, so
                // that entire period should be idle — but previous polls
                // already credited it as active.  Retroactively transfer the
                // backlog from active → idle so totals are accurate.
                let idle_secs_i64 = idle_secs_system as i64;

                // Current poll slice: user was idle for the whole thing
                // (last input was longer ago than `elapsed`).
                let active_slice = (elapsed - idle_secs_i64).max(0);
                let idle_slice   = elapsed - active_slice;

                // Backlog: time before this poll that was already credited
                // as active but actually belongs to idle.
                let backlog = (idle_secs_i64 - elapsed).max(0);
                let backlog = backlog.min(t.current_active_secs); // never go negative
                t.current_active_secs -= backlog;
                t.current_idle_secs   += backlog;

                t.current_active_secs += active_slice;
                t.current_idle_secs   += idle_slice;

                t.session_start = now - chrono::Duration::seconds(idle_secs_i64);
                t.status = "idle".to_string();
                poll_count = 0;
            } else if t.status != "productive" && idle_secs_system < threshold {
                // idle or locked → productive.
                // Split the current poll slice at the last-input boundary so
                // only the tail (where user was already active) goes to active.
                let idle_secs_i64 = idle_secs_system as i64;
                let active_part   = idle_secs_i64.min(elapsed);
                let idle_part     = elapsed - active_part;

                match t.status.as_str() {
                    "idle"   => {
                        t.current_idle_secs   += idle_part;
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
                // still idle (threshold-based)
                t.current_idle_secs += elapsed;
            } else {
                // locked but lock API says unlocked yet idle_secs >= threshold:
                // treat as locked until confirmed productive.
                t.current_locked_secs += elapsed;
            }

            // ── Checkpoint: persist accumulated counters ──────────────────────
            if poll_count >= CHECKPOINT_POLLS {
                if let Ok(conn) = db.lock() {
                    let _ = crate::db::update_session_time(
                        &conn,
                        t.current_session_id,
                        t.current_active_secs,
                        t.current_idle_secs,
                        t.current_locked_secs,
                    );
                }
                poll_count = 0;
            }

            if t.status != prev_status { Some(t.status.clone()) } else { None }
        };
        if let Some(status) = new_status {
            update_tray_icon(&app_handle, &status);
            // Emit immediately so the frontend can react without waiting
            // for its next poll cycle.
            let _ = app_handle.emit("tracker-status-changed", &status);
        }

        // ── App usage tracking ────────────────────────────────────────────────
        // Sample the foreground app every poll cycle while the user is active.
        {
            let current_status = {
                let t = tracker.lock().unwrap();
                t.status.clone()
            };
            if current_status == "productive" {
                let app_elapsed = (now - last_app_tick).num_seconds().max(0).min(MAX_ELAPSED_SECS);
                if app_elapsed > 0 {
                    let active_app = crate::active_app::get_active_app();
                    let local_date = now
                        .with_timezone(&Local)
                        .format("%Y-%m-%d")
                        .to_string();
                    if let Ok(conn) = db.lock() {
                        let _ = crate::db::upsert_app_usage(
                            &conn,
                            &active_app.name,
                            &local_date,
                            app_elapsed,
                            &active_app.exe_path,
                        );
                    }
                }
            }
            last_app_tick = now;
        }
    }
}
