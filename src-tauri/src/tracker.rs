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

/// Real-time gap between polls (seconds) that indicates the system was
/// suspended (sleep / hibernate / deep-idle).  60 s = 2× the poll interval,
/// so normal timer jitter will never trigger a false positive.
const SUSPEND_GAP_SECS: i64 = 60;

/// Polls every 30 seconds, transitions between productive / idle states, and
/// writes **checkpoint segments** every 5 minutes so data is reliably stored
/// even when the user stays in the same state for a long time or the app is
/// killed between transitions.
///
/// # Sleep / lock handling
/// When the computer is suspended (sleep, hibernate) or the screen is locked
/// long enough, the missing wall-clock time is recorded as `"idle"` so it is
/// never counted as productive time:
/// - **Screen lock**: `GetLastInputInfo` accumulates idle time normally while
///   the screen is locked, so the regular idle-threshold logic already handles
///   this case.
/// - **Sleep / suspend**: the tokio timer and `GetTickCount` both pause during
///   suspension, so the elapsed wall-clock time would otherwise be silently
///   lost.  We detect this by comparing successive `Utc::now()` values; when
///   the real gap exceeds `SUSPEND_GAP_SECS` we flush the pre-suspend segment
///   and write the entire gap as an `"idle"` segment.
pub async fn run_tracker(db: Arc<Mutex<Connection>>, tracker: Arc<Mutex<TrackerShared>>) {
    let mut poll_count: u32 = 0;
    // Wall-clock timestamp of the previous poll, used to detect system suspend.
    let mut last_poll_wall = Utc::now();

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;

        let now = Utc::now();
        let actual_gap_secs = (now - last_poll_wall).num_seconds();
        last_poll_wall = now;

        // ── Suspend / sleep / hibernate detection ────────────────────────────
        // If the real-time gap between polls is much larger than 30 s the
        // system was suspended.  Flush whatever was in progress and record the
        // entire gap as idle so it is never attributed to productive time.
        if actual_gap_secs > SUSPEND_GAP_SECS {
            let sleep_start = now - chrono::Duration::seconds(actual_gap_secs);

            let segments: Vec<(DateTime<Utc>, DateTime<Utc>, String)> = {
                let mut t = tracker.lock().unwrap();
                let mut segs = Vec::new();

                // Flush the segment that was in progress before suspension.
                if sleep_start > t.session_start {
                    segs.push((t.session_start, sleep_start, t.status.clone()));
                }

                // The suspension window itself is non-productive time.
                segs.push((sleep_start, now, "idle".to_string()));

                // Resume in idle state; user input will trigger the → productive
                // transition on the next poll.
                t.session_start = now;
                t.status = "idle".to_string();
                poll_count = 0;
                segs
            };

            if let Ok(conn) = db.lock() {
                for (start, end, stype) in segments {
                    let _ = crate::db::insert_session(&conn, &start, &end, &stype);
                }
            }

            continue; // skip the normal idle-detection logic this poll
        }

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
