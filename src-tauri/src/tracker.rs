use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use rusqlite::Connection;

// ── Shared tracker state ──────────────────────────────────────────────────────

pub struct TrackerShared {
    /// "productive" | "idle"
    pub status: String,
    /// UTC timestamp when the current session began
    pub session_start: DateTime<Utc>,
    /// Seconds of inactivity that triggers the productive → idle transition
    pub idle_threshold_secs: u64,
}

// ── Background task ───────────────────────────────────────────────────────────

/// How many 30-second polls before force-flushing the current segment.
/// 10 × 30 s = 5 minutes — limits data loss if the process is killed.
const CHECKPOINT_POLLS: u32 = 10;

/// Polls every 30 seconds, transitions between productive / idle states, and
/// writes **checkpoint segments** every 5 minutes so data is reliably stored
/// even when the user stays in the same state for a long time or the app is
/// killed between transitions.
pub async fn run_tracker(db: Arc<Mutex<Connection>>, tracker: Arc<Mutex<TrackerShared>>) {
    let mut poll_count: u32 = 0;

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        poll_count += 1;

        let idle_secs = crate::idle::get_idle_seconds();
        let now = Utc::now();

        // ── Evaluate transition or periodic checkpoint ────────────────────────
        let to_write: Option<(DateTime<Utc>, DateTime<Utc>, String)> = {
            let mut t = tracker.lock().unwrap();
            let threshold = t.idle_threshold_secs;

            if t.status == "productive" && idle_secs >= threshold {
                // Transition: productive → idle
                let last_input = now - chrono::Duration::seconds(idle_secs as i64);
                let end = if last_input > t.session_start { last_input } else { t.session_start };
                let session = (t.session_start, end, "productive".to_string());
                t.session_start = end;
                t.status = "idle".to_string();
                poll_count = 0;
                Some(session)
            } else if t.status == "idle" && idle_secs < threshold {
                // Transition: idle → productive
                let comeback = now - chrono::Duration::seconds(idle_secs as i64);
                let end = if comeback > t.session_start { comeback } else { now };
                let session = (t.session_start, end, "idle".to_string());
                t.session_start = end;
                t.status = "productive".to_string();
                poll_count = 0;
                Some(session)
            } else if poll_count >= CHECKPOINT_POLLS {
                // No state change — checkpoint the segment so it reaches disk.
                let session = (t.session_start, now, t.status.clone());
                t.session_start = now;
                poll_count = 0;
                Some(session)
            } else {
                None
            }
        }; // mutex released here

        // ── Persist segment ───────────────────────────────────────────────────
        if let Some((start, end, stype)) = to_write {
            if let Ok(conn) = db.lock() {
                let _ = crate::db::insert_session(&conn, &start, &end, &stype);
            }
        }
    }
}
