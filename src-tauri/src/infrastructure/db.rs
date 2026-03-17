use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result as SqlResult};
use uuid::Uuid;

use crate::domain::{
    entities::{AppUsageStat, DailySummary, Interval, Session},
    ports::{AppUsageRepository, SessionRepository, SettingsRepository},
};

// ── Concrete repository ───────────────────────────────────────────────────────

/// Wraps a shared SQLite connection and implements all persistence port traits.
/// All three repository traits are implemented on this single struct so the
/// same underlying connection is reused across every operation.
pub struct SqliteDb {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteDb {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: Arc::new(Mutex::new(conn)),
        }
    }
}

// ── SessionRepository ─────────────────────────────────────────────────────────

impl SessionRepository for SqliteDb {
    fn begin_session(&self, date: &str, start: &DateTime<Utc>) -> Result<String, String> {
        let id = Uuid::new_v4().to_string();
        let start_str = start.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let conn = self.conn.lock().unwrap();
        // Upsert: if a session for this date already exists, keep it as-is and
        // return its existing id; otherwise insert a fresh row.
        conn.execute(
            "INSERT INTO sessions (id, date, start_time) VALUES (?1, ?2, ?3)
             ON CONFLICT(date) DO NOTHING",
            params![id, date, start_str],
        )
        .map_err(|e| e.to_string())?;
        // Always return the id that is actually stored for this date.
        let stored_id: String = conn
            .query_row(
                "SELECT id FROM sessions WHERE date = ?1",
                params![date],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        Ok(stored_id)
    }

    fn update_session_time(
        &self,
        id: &str,
        active_secs: i64,
        idle_secs: i64,
        locked_secs: i64,
    ) -> Result<(), String> {
        let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let conn = self.conn.lock().unwrap();
        for (itype, secs) in [("active", active_secs), ("idle", idle_secs), ("locked", locked_secs)] {
            let interval_id = format!("{}-{}", id, itype);
            conn.execute(
                "INSERT INTO intervals (id, session_id, type, start_time, duration_secs) \
                 VALUES (?1, ?2, ?3, ?4, ?5) \
                 ON CONFLICT(id) DO UPDATE SET duration_secs = excluded.duration_secs",
                params![interval_id, id, itype, now, secs],
            )
            .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn end_session(&self, id: &str, end: &DateTime<Utc>) -> Result<(), String> {
        let end_str = end.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE sessions SET end_time = ?1 WHERE id = ?2",
            params![end_str, id],
        )
        .map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE intervals SET end_time = ?1 WHERE session_id = ?2 AND end_time IS NULL",
            params![end_str, id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_session_for_date(&self, date: &str) -> Result<Option<Session>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT session_id, date, start_time, end_time,
                        active_secs, idle_secs, locked_secs, unknown_secs
                 FROM session_stats
                 WHERE date = ?1",
            )
            .map_err(|e| e.to_string())?;

        let mut rows = stmt
            .query_map(params![date], |row| {
                Ok(Session {
                    id: row.get(0)?,
                    date: row.get(1)?,
                    start_time: row.get(2)?,
                    end_time: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    active_secs: row.get(4)?,
                    idle_secs: row.get(5)?,
                    locked_secs: row.get(6)?,
                    unknown_secs: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?;

        match rows.next() {
            None => Ok(None),
            Some(r) => r.map(Some).map_err(|e| e.to_string()),
        }
    }

    fn get_today_stats(&self, date: &str) -> Result<(i64, i64, i64), String> {
        let conn = self.conn.lock().unwrap();
        let row: (i64, i64, i64) = conn
            .query_row(
                "SELECT COALESCE(active_secs, 0),
                        COALESCE(idle_secs,   0),
                        COALESCE(locked_secs, 0)
                 FROM session_stats
                 WHERE date = ?1",
                params![date],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap_or((0, 0, 0));
        Ok(row)
    }

    fn get_history(&self, days: u32) -> Result<Vec<DailySummary>, String> {
        let conn = self.conn.lock().unwrap();
        let offset = format!("-{} days", days.saturating_sub(1));

        let mut stmt = conn
            .prepare(
                "SELECT
                     date,
                     COALESCE(active_secs,  0) AS prod_secs,
                     COALESCE(idle_secs,    0) AS idle_secs,
                     COALESCE(locked_secs,  0) AS locked_secs
                 FROM session_stats
                 WHERE date >= date('now', 'localtime', ?1)
                 ORDER BY date DESC",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![offset], |row| {
                Ok(DailySummary {
                    date: row.get(0)?,
                    productive_secs: row.get(1)?,
                    idle_secs: row.get(2)?,
                    locked_secs: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?;

        rows.collect::<SqlResult<Vec<_>>>().map_err(|e| e.to_string())
    }

    fn get_intervals_for_session(&self, session_id: &str) -> Result<Vec<Interval>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, app_usage_id, start_time, end_time, duration_secs, type
                 FROM intervals
                 WHERE session_id = ?1
                 ORDER BY start_time ASC",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![session_id], |row| {
                Ok(Interval {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    app_usage_id: row.get(2)?,
                    start_time: row.get(3)?,
                    end_time: row.get(4)?,
                    duration_secs: row.get(5)?,
                    interval_type: row.get(6)?,
                })
            })
            .map_err(|e| e.to_string())?;

        rows.collect::<SqlResult<Vec<_>>>().map_err(|e| e.to_string())
    }

    fn clear_all_data(&self) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "DELETE FROM intervals;
             DELETE FROM sessions;
             DELETE FROM app_usage;",
        )
        .map_err(|e| e.to_string())
    }
}

// ── AppUsageRepository ────────────────────────────────────────────────────────

impl AppUsageRepository for SqliteDb {
    fn upsert_app_usage(
        &self,
        app_name: &str,
        date: &str,
        duration_secs: i64,
        exe_path: &str,
    ) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO app_usage (app_name, date, duration_secs, exe_path) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(app_name, date) DO UPDATE SET
                 duration_secs = duration_secs + excluded.duration_secs,
                 exe_path = CASE WHEN excluded.exe_path != '' THEN excluded.exe_path ELSE exe_path END",
            params![app_name, date, duration_secs, exe_path],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_app_usage_for_date(&self, date: &str) -> Result<Vec<AppUsageStat>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT app_name, duration_secs, COALESCE(exe_path, ''), pct_of_day
                 FROM app_usage_by_date
                 WHERE date = ?1",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![date], |row| {
                Ok(AppUsageStat {
                    app_name: row.get(0)?,
                    duration_secs: row.get(1)?,
                    exe_path: row.get(2)?,
                    pct_of_day: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?;

        rows.collect::<SqlResult<Vec<_>>>().map_err(|e| e.to_string())
    }

    fn get_exe_path_for_app(&self, app_name: &str) -> Result<Option<String>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT exe_path FROM app_usage WHERE app_name = ?1 AND exe_path != '' LIMIT 1",
            )
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![app_name]).map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            Ok(Some(row.get(0).map_err(|e| e.to_string())?))
        } else {
            Ok(None)
        }
    }

    fn clear_exe_path_cache(&self) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE app_usage SET exe_path = ''", [])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

// ── SettingsRepository ────────────────────────────────────────────────────────

impl SettingsRepository for SqliteDb {
    fn get_setting(&self, key: &str) -> Result<Option<String>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT value FROM settings WHERE key = ?1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![key]).map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            Ok(Some(row.get(0).map_err(|e| e.to_string())?))
        } else {
            Ok(None)
        }
    }

    fn set_setting(&self, key: &str, value: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}
