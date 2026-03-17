use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result};
use serde::Serialize;

// ── Data types ──────────────────────────────────────────────────────────────

/// One app session: from when the tracker starts (or resumes after suspend) to
/// when it stops (app quit / shutdown / suspend).  Time within the session is
/// split into `active_secs` (productive), `idle_secs` (threshold-based
/// inactivity), and `locked_secs` (screen-lock).
#[derive(Debug, Serialize, Clone)]
pub struct Session {
    pub id: i64,
    pub start_time: String,   // ISO 8601 UTC
    pub end_time: String,     // ISO 8601 UTC, empty string = in-progress
    pub active_secs: i64,
    pub idle_secs: i64,
    pub locked_secs: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct DailySummary {
    pub date: String,            // "YYYY-MM-DD" local date
    pub productive_secs: i64,
    pub idle_secs: i64,
    pub locked_secs: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct AppUsageStat {
    pub app_name: String,
    pub duration_secs: i64,
    pub exe_path: String,
}

// ── Session CRUD ──────────────────────────────────────────────────────────────

/// Opens a new in-progress session and returns its DB row id.
pub fn begin_session(conn: &Connection, start: &DateTime<Utc>) -> Result<i64> {
    conn.execute(
        "INSERT INTO sessions (start_time, active_secs, idle_secs, locked_secs) VALUES (?1, 0, 0, 0)",
        params![start.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Persists the accumulated active/idle/locked counters for the in-progress session.
pub fn update_session_time(
    conn: &Connection,
    id: i64,
    active_secs: i64,
    idle_secs: i64,
    locked_secs: i64,
) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET active_secs = ?1, idle_secs = ?2, locked_secs = ?3 WHERE id = ?4",
        params![active_secs, idle_secs, locked_secs, id],
    )?;
    Ok(())
}

/// Closes an in-progress session by setting its end_time.
pub fn end_session(conn: &Connection, id: i64, end: &DateTime<Utc>) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET end_time = ?1 WHERE id = ?2",
        params![end.to_rfc3339_opts(chrono::SecondsFormat::Secs, true), id],
    )?;
    Ok(())
}

// ── Query helpers ─────────────────────────────────────────────────────────────

/// Returns (active_secs, idle_secs, locked_secs) for completed sessions on a
/// given local date "YYYY-MM-DD".  The caller adds the in-progress counters.
pub fn get_today_stats(conn: &Connection, local_today: &str) -> Result<(i64, i64, i64)> {
    let row: (i64, i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(active_secs), 0),
                COALESCE(SUM(idle_secs),   0),
                COALESCE(SUM(locked_secs), 0)
         FROM sessions
         WHERE date(datetime(start_time, 'localtime')) = ?1
           AND end_time IS NOT NULL",
        params![local_today],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;
    Ok(row)
}

/// Returns all completed sessions for a given local date "YYYY-MM-DD".
pub fn get_sessions_for_date(conn: &Connection, local_date: &str) -> Result<Vec<Session>> {
    let mut stmt = conn.prepare(
        "SELECT id, start_time, end_time, active_secs, idle_secs, locked_secs
         FROM sessions
         WHERE date(datetime(start_time, 'localtime')) = ?1
         ORDER BY start_time ASC",
    )?;

    let rows = stmt.query_map(params![local_date], |row| {
        Ok(Session {
            id: row.get(0)?,
            start_time: row.get(1)?,
            end_time: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            active_secs: row.get(3)?,
            idle_secs: row.get(4)?,
            locked_secs: row.get(5)?,
        })
    })?;

    rows.collect()
}

/// Returns daily summaries for the last `days` days (local timezone), newest first.
pub fn get_history(conn: &Connection, days: u32) -> Result<Vec<DailySummary>> {
    let offset = format!("-{} days", days.saturating_sub(1));

    let mut stmt = conn.prepare(
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
    )?;

    let rows = stmt.query_map(params![offset], |row| {
        Ok(DailySummary {
            date: row.get(0)?,
            productive_secs: row.get(1)?,
            idle_secs: row.get(2)?,
            locked_secs: row.get(3)?,
        })
    })?;

    rows.collect()
}

// ── App usage ─────────────────────────────────────────────────────────────────

/// Adds `duration_secs` to the running total for (app_name, date) and stores
/// `exe_path` (kept from the first non-empty value seen for that app).
/// Creates the row if it does not yet exist.
pub fn upsert_app_usage(
    conn: &Connection,
    app_name: &str,
    date: &str,
    duration_secs: i64,
    exe_path: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO app_usage (app_name, date, duration_secs, exe_path) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(app_name, date) DO UPDATE SET
             duration_secs = duration_secs + excluded.duration_secs,
             exe_path = CASE WHEN excluded.exe_path != '' THEN excluded.exe_path ELSE exe_path END",
        params![app_name, date, duration_secs, exe_path],
    )?;
    Ok(())
}

/// Returns the stored exe path for the given app name, or `None` when unknown.
pub fn get_exe_path_for_app(conn: &Connection, app_name: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare(
        "SELECT exe_path FROM app_usage WHERE app_name = ?1 AND exe_path != '' LIMIT 1",
    )?;
    let mut rows = stmt.query(params![app_name])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

/// Clears all stored exe paths so they will be re-discovered on the next poll
/// or icon request.  Useful after an app update moves its executable.
pub fn clear_exe_path_cache(conn: &Connection) -> Result<()> {
    conn.execute("UPDATE app_usage SET exe_path = ''", [])?;
    Ok(())
}

/// Returns all apps with accumulated time for a given local date, sorted by
/// duration descending.
pub fn get_app_usage_for_date(conn: &Connection, date: &str) -> Result<Vec<AppUsageStat>> {
    let mut stmt = conn.prepare(
        "SELECT app_name, duration_secs, COALESCE(exe_path, '') FROM app_usage
         WHERE date = ?1
         ORDER BY duration_secs DESC",
    )?;

    let rows = stmt.query_map(params![date], |row| {
        Ok(AppUsageStat {
            app_name: row.get(0)?,
            duration_secs: row.get(1)?,
            exe_path: row.get(2)?,
        })
    })?;

    rows.collect()
}

// ── Settings ──────────────────────────────────────────────────────────────────

pub fn get_setting(conn: &Connection, key: &str) -> Result<String> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    )?;
    Ok(())
}

// ── Data wipe ─────────────────────────────────────────────────────────────────

/// Deletes all sessions and app-usage rows, preserving settings.
pub fn clear_all_data(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "DELETE FROM sessions;
         DELETE FROM app_usage;",
    )
}
