-- One session per local date. start_time records first boot of that day.
CREATE TABLE IF NOT EXISTS sessions (
    id          TEXT  PRIMARY KEY,
    date        TEXT  NOT NULL UNIQUE,  -- "YYYY-MM-DD" local date
    start_time  TEXT  NOT NULL,
    end_time    TEXT
);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS app_usage (
    app_name      TEXT    NOT NULL,
    date          TEXT    NOT NULL,
    duration_secs INTEGER NOT NULL DEFAULT 0,
    exe_path      TEXT    NOT NULL DEFAULT '',
    PRIMARY KEY (app_name, date)
);

-- Step 2: Intervals table — one cumulative row per (session, type).
--   duration_secs is upserted on every checkpoint; start_time/end_time
--   record when the interval was first opened and when the session closed.

-- Intervals: each row is a time-slice for a given type within a session.
--   Multiple rows per type are allowed (one per status-change event).
CREATE TABLE IF NOT EXISTS intervals (
    id            TEXT    PRIMARY KEY,
    session_id    TEXT    NOT NULL,
    app_usage_id  TEXT,
    start_time    TEXT    NOT NULL,
    end_time      TEXT,
    duration_secs INTEGER NOT NULL DEFAULT 0,
    type          TEXT    NOT NULL CHECK (type IN ('active', 'idle', 'locked', 'unknown')),
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

-- session_stats view: aggregated totals per session.
CREATE VIEW IF NOT EXISTS session_stats AS
SELECT
    s.id                                                                            AS session_id,
    s.date,
    s.start_time,
    s.end_time,
    COALESCE(SUM(CASE WHEN i.type = 'active'  THEN i.duration_secs ELSE 0 END), 0) AS active_secs,
    COALESCE(SUM(CASE WHEN i.type = 'idle'    THEN i.duration_secs ELSE 0 END), 0) AS idle_secs,
    COALESCE(SUM(CASE WHEN i.type = 'locked'  THEN i.duration_secs ELSE 0 END), 0) AS locked_secs,
    COALESCE(SUM(CASE WHEN i.type = 'unknown' THEN i.duration_secs ELSE 0 END), 0) AS unknown_secs
FROM sessions s
LEFT JOIN intervals i ON i.session_id = s.id
GROUP BY s.id;

-- app_usage_by_date view.
CREATE VIEW IF NOT EXISTS app_usage_by_date AS
SELECT
    date,
    app_name,
    duration_secs,
    exe_path,
    ROUND(
        100.0 * duration_secs / SUM(duration_secs) OVER (PARTITION BY date),
        1
    ) AS pct_of_day
FROM app_usage
ORDER BY date DESC, duration_secs DESC;

INSERT OR IGNORE INTO settings (key, value) VALUES ('idle_threshold_mins', '5');
INSERT OR IGNORE INTO settings (key, value) VALUES ('autostart', 'false');
