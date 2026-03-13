use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result};
use serde::Serialize;

// ── Data types ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct Session {
    pub id: i64,
    pub start_time: String,
    pub end_time: String,        // empty string = in-progress
    pub session_type: String,    // "productive" | "idle"
    pub duration_secs: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct DailySummary {
    pub date: String,            // "YYYY-MM-DD" local date
    pub productive_secs: i64,
    pub idle_secs: i64,
}

// ── Schema init ──────────────────────────────────────────────────────────────

pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sessions (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            start_time   TEXT NOT NULL,
            end_time     TEXT,
            session_type TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
         );
         INSERT OR IGNORE INTO settings (key, value) VALUES ('idle_threshold_mins', '5');
         INSERT OR IGNORE INTO settings (key, value) VALUES ('autostart', 'false');",
    )
}

// ── Session CRUD ──────────────────────────────────────────────────────────────

/// Insert a completed session. Skips sessions shorter than 5 seconds.
pub fn insert_session(
    conn: &Connection,
    start: &DateTime<Utc>,
    end: &DateTime<Utc>,
    session_type: &str,
) -> Result<()> {
    if (*end - *start).num_seconds() < 5 {
        return Ok(());
    }
    conn.execute(
        "INSERT INTO sessions (start_time, end_time, session_type) VALUES (?1, ?2, ?3)",
        params![
            start.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            end.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            session_type,
        ],
    )?;
    Ok(())
}

// ── Query helpers ─────────────────────────────────────────────────────────────

/// Returns (productive_secs, idle_secs) for a given local date string "YYYY-MM-DD".
/// Durations are computed from completed sessions only.
pub fn get_today_stats(conn: &Connection, local_today: &str) -> Result<(i64, i64)> {
    let prod: i64 = conn.query_row(
        "SELECT COALESCE(SUM(
             CAST(strftime('%s', end_time) AS INTEGER)
             - CAST(strftime('%s', start_time) AS INTEGER)
         ), 0)
         FROM sessions
         WHERE session_type = 'productive'
           AND date(datetime(start_time, 'localtime')) = ?1
           AND end_time IS NOT NULL",
        params![local_today],
        |row| row.get(0),
    )?;

    let idle: i64 = conn.query_row(
        "SELECT COALESCE(SUM(
             CAST(strftime('%s', end_time) AS INTEGER)
             - CAST(strftime('%s', start_time) AS INTEGER)
         ), 0)
         FROM sessions
         WHERE session_type = 'idle'
           AND date(datetime(start_time, 'localtime')) = ?1
           AND end_time IS NOT NULL",
        params![local_today],
        |row| row.get(0),
    )?;

    Ok((prod, idle))
}

/// Returns all completed sessions for a given local date string "YYYY-MM-DD".
pub fn get_sessions_for_date(conn: &Connection, local_date: &str) -> Result<Vec<Session>> {
    let mut stmt = conn.prepare(
        "SELECT id, start_time, end_time, session_type,
                CAST(strftime('%s', end_time) AS INTEGER)
                - CAST(strftime('%s', start_time) AS INTEGER) AS duration_secs
         FROM sessions
         WHERE date(datetime(start_time, 'localtime')) = ?1
         ORDER BY start_time ASC",
    )?;

    let rows = stmt.query_map(params![local_date], |row| {
        Ok(Session {
            id: row.get(0)?,
            start_time: row.get(1)?,
            end_time: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            session_type: row.get(3)?,
            duration_secs: row.get::<_, Option<i64>>(4)?.unwrap_or(0),
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
             COALESCE(SUM(CASE WHEN session_type = 'productive'
                 THEN CAST(strftime('%s', end_time) AS INTEGER)
                      - CAST(strftime('%s', start_time) AS INTEGER)
                 ELSE 0 END), 0) AS prod_secs,
             COALESCE(SUM(CASE WHEN session_type = 'idle'
                 THEN CAST(strftime('%s', end_time) AS INTEGER)
                      - CAST(strftime('%s', start_time) AS INTEGER)
                 ELSE 0 END), 0) AS idle_secs
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
