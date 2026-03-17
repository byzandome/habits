use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result as SqlResult};

use crate::domain::{
    entities::{AppUsageStat, DailySummary, Session},
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
    fn begin_session(&self, start: &DateTime<Utc>) -> Result<i64, String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO sessions (start_time, active_secs, idle_secs, locked_secs) VALUES (?1, 0, 0, 0)",
            params![start.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)],
        )
        .map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    }

    fn update_session_time(
        &self,
        id: i64,
        active_secs: i64,
        idle_secs: i64,
        locked_secs: i64,
    ) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE sessions SET active_secs = ?1, idle_secs = ?2, locked_secs = ?3 WHERE id = ?4",
            params![active_secs, idle_secs, locked_secs, id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn end_session(&self, id: i64, end: &DateTime<Utc>) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE sessions SET end_time = ?1 WHERE id = ?2",
            params![
                end.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                id
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_sessions_for_date(&self, date: &str) -> Result<Vec<Session>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, start_time, end_time, active_secs, idle_secs, locked_secs
                 FROM sessions
                 WHERE date(datetime(start_time, 'localtime')) = ?1
                 ORDER BY start_time ASC",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![date], |row| {
                Ok(Session {
                    id: row.get(0)?,
                    start_time: row.get(1)?,
                    end_time: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    active_secs: row.get(3)?,
                    idle_secs: row.get(4)?,
                    locked_secs: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;

        rows.collect::<SqlResult<Vec<_>>>().map_err(|e| e.to_string())
    }

    fn get_today_stats(&self, date: &str) -> Result<(i64, i64, i64), String> {
        let conn = self.conn.lock().unwrap();
        let row: (i64, i64, i64) = conn
            .query_row(
                "SELECT COALESCE(SUM(active_secs), 0),
                        COALESCE(SUM(idle_secs),   0),
                        COALESCE(SUM(locked_secs), 0)
                 FROM sessions
                 WHERE date(datetime(start_time, 'localtime')) = ?1
                   AND end_time IS NOT NULL",
                params![date],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| e.to_string())?;
        Ok(row)
    }

    fn get_history(&self, days: u32) -> Result<Vec<DailySummary>, String> {
        let conn = self.conn.lock().unwrap();
        let offset = format!("-{} days", days.saturating_sub(1));

        let mut stmt = conn
            .prepare(
                "SELECT
                     date(datetime(start_time, 'localtime')) AS day,
                     COALESCE(SUM(active_secs),  0) AS prod_secs,
                     COALESCE(SUM(idle_secs),    0) AS idle_secs,
                     COALESCE(SUM(locked_secs),  0) AS locked_secs
                 FROM sessions
                 WHERE end_time IS NOT NULL
                   AND date(datetime(start_time, 'localtime'))
                       >= date('now', 'localtime', ?1)
                 GROUP BY day
                 ORDER BY day DESC",
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

    fn clear_all_data(&self) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "DELETE FROM sessions;
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
                "SELECT app_name, duration_secs, COALESCE(exe_path, '') FROM app_usage
                 WHERE date = ?1
                 ORDER BY duration_secs DESC",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![date], |row| {
                Ok(AppUsageStat {
                    app_name: row.get(0)?,
                    duration_secs: row.get(1)?,
                    exe_path: row.get(2)?,
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
