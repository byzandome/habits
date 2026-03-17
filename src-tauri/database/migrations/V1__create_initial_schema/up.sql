CREATE TABLE IF NOT EXISTS sessions (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    start_time   TEXT    NOT NULL,
    end_time     TEXT,
    active_secs  INTEGER NOT NULL DEFAULT 0,
    idle_secs    INTEGER NOT NULL DEFAULT 0,
    locked_secs  INTEGER NOT NULL DEFAULT 0
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

INSERT OR IGNORE INTO settings (key, value) VALUES ('idle_threshold_mins', '5');
INSERT OR IGNORE INTO settings (key, value) VALUES ('autostart', 'false');
