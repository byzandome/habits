CREATE TABLE settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT
);
CREATE TABLE apps (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    path TEXT NOT NULL UNIQUE,
    color TEXT
);
CREATE TABLE app_usages (
    id TEXT PRIMARY KEY NOT NULL,
    start_at TEXT NOT NULL,
    duration_secs INTEGER,
    end_at TEXT,
    app_id TEXT REFERENCES apps(id) DEFERRABLE INITIALLY DEFERRED
);
CREATE TABLE domains (
    id TEXT PRIMARY KEY NOT NULL,
    url TEXT NOT NULL,
    name TEXT
);
CREATE TABLE domain_history (
    id TEXT PRIMARY KEY NOT NULL,
    domain_id TEXT NOT NULL REFERENCES domains(id) DEFERRABLE INITIALLY DEFERRED,
    url TEXT NOT NULL,
    start_at TEXT NOT NULL,
    end_at TEXT,
    duration_secs INTEGER
);
-- Default settings
INSERT
    OR IGNORE INTO settings (key, value)
VALUES ('idle_threshold_mins', '5');
INSERT
    OR IGNORE INTO settings (key, value)
VALUES ('autostart', 'false');
INSERT
    OR IGNORE INTO settings (key, value)
VALUES ('theme', 'system');
INSERT
    OR IGNORE INTO settings (key, value)
VALUES ('lang', 'en');