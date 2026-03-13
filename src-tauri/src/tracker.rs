use std::sync::{Arc, Mutex};

use chrono::{DateTime, Local, Utc};
use rusqlite::Connection;

// ── Shared tracker state ──────────────────────────────────────────────────────

pub struct TrackerShared {
    /// "productive" | "idle" — current user-activity state.
    pub status: String,
    /// UTC timestamp when the current productive/idle state began (used for the
    /// live session-duration display in the UI).
    pub session_start: DateTime<Utc>,
    /// Seconds of inactivity that triggers the productive → idle transition.
    pub idle_threshold_secs: u64,
    /// DB row id of the currently open app session (–1 while not yet inserted).
    pub current_session_id: i64,
    /// Active seconds accumulated so far in the current app session.
    pub current_active_secs: i64,
    /// Idle seconds accumulated so far in the current app session.
    pub current_idle_secs: i64,
}

// ── Background task ───────────────────────────────────────────────────────────

/// How many 30-second polls before force-writing the accumulated counters to DB.
/// 10 × 30 s = 5 minutes — limits data loss if the process is killed.
const CHECKPOINT_POLLS: u32 = 10;

/// Real-time gap between polls (seconds) that indicates the system was
/// suspended.  60 s = 2× the poll interval, so normal jitter never fires this.
const SUSPEND_GAP_SECS: i64 = 60;

/// Maximum seconds credited to a single poll cycle.  Guards against false
/// accumulation when the timer wakes up late for any reason other than suspend.
const MAX_ELAPSED_SECS: i64 = 60;

/// Polls every 30 seconds.
///
/// **Session model (Teams-style):**  
/// One *app session* spans from the moment the tracker starts (or the system
/// resumes after suspend) until the app is quit or the system suspends again.
/// Within a session every 30-second slice is added to `active_secs` when the
/// user was interacting with the PC, or to `idle_secs` when the idle time
/// exceeds the configured threshold.  No new session row is created merely
/// because the user went idle.
///
/// **Suspend / sleep / hibernate handling:**  
/// When the real wall-clock gap between two polls is larger than
/// `SUSPEND_GAP_SECS` we know the system was suspended.  We close the
/// pre-suspend session and immediately open a fresh one, so the suspension
/// period is never silently attributed to either active or idle time.
pub async fn run_tracker(db: Arc<Mutex<Connection>>, tracker: Arc<Mutex<TrackerShared>>) {
    let mut poll_count: u32 = 0;
    let mut last_poll_wall = Utc::now();
    let mut last_app_tick = Utc::now();

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;

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
                poll_count = 0;
            }

            last_app_tick = now;
            continue; // skip normal idle-detection this poll
        }

        poll_count += 1;

        let idle_secs_system = crate::idle::get_idle_seconds();
        let now = Utc::now();

        // Elapsed since last poll, capped to guard against stale timers.
        let elapsed = actual_gap_secs.max(0).min(MAX_ELAPSED_SECS);

        // ── Accumulate time & detect productive ↔ idle transition ─────────────
        {
            let mut t = tracker.lock().unwrap();
            let threshold = t.idle_threshold_secs;

            if t.status == "productive" && idle_secs_system >= threshold {
                // The user went idle.  Credit the slice up to the last-input
                // moment as active, the rest as idle.
                let idle_start_offset = idle_secs_system as i64;
                let active_slice = (elapsed - idle_start_offset).max(0);
                let idle_slice = elapsed - active_slice;

                t.current_active_secs += active_slice;
                t.current_idle_secs += idle_slice;

                t.session_start = now - chrono::Duration::seconds(idle_secs_system as i64);
                t.status = "idle".to_string();
                poll_count = 0; // force a checkpoint after every transition
            } else if t.status == "idle" && idle_secs_system < threshold {
                // User came back.
                t.current_idle_secs += elapsed;
                t.session_start = now - chrono::Duration::seconds(idle_secs_system as i64);
                t.status = "productive".to_string();
                poll_count = 0;
            } else if t.status == "productive" {
                t.current_active_secs += elapsed;
            } else {
                // still idle
                t.current_idle_secs += elapsed;
            }

            // ── Checkpoint: persist accumulated counters ──────────────────────
            if poll_count >= CHECKPOINT_POLLS {
                if let Ok(conn) = db.lock() {
                    let _ = crate::db::update_session_time(
                        &conn,
                        t.current_session_id,
                        t.current_active_secs,
                        t.current_idle_secs,
                    );
                }
                poll_count = 0;
            }
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
                    let app_name = crate::active_app::get_active_app();
                    let local_date = now
                        .with_timezone(&Local)
                        .format("%Y-%m-%d")
                        .to_string();
                    if let Ok(conn) = db.lock() {
                        let _ = crate::db::upsert_app_usage(&conn, &app_name, &local_date, app_elapsed);
                    }
                }
            }
            last_app_tick = now;
        }
    }
}
