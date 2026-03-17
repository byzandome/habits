mod application;
mod domain;
mod infrastructure;
mod presentation;

use std::sync::{Arc, Mutex};

use chrono::Utc;
use rusqlite::Connection;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
use tauri_plugin_sql::{Migration, MigrationKind};

use domain::entities::TrackerState;
use domain::ports::{SessionRepository, SettingsRepository};
use infrastructure::db::SqliteDb;

// ── Shared application state ──────────────────────────────────────────────────

/// Holds the concrete implementations injected at startup.
/// Commands receive this via Tauri's managed-state mechanism.
pub struct AppState {
    pub db: Arc<SqliteDb>,
    pub tracker: Arc<Mutex<TrackerState>>,
}

// Arc<Mutex<T>> is Send + Sync when T: Send, so Tauri can hold AppState safely.

// ── Entry point ───────────────────────────────────────────────────────────────

// ── Database migrations ─────────────────────────────────────────────────────
//
// SQL lives in database/migrations/.  Naming convention:
//   V{version}__{description}/up.sql   — applied going forward
//   V{version}__{description}/down.sql — applied when rolling back (optional)
//
// RULES:
//   - Never modify or remove an existing file/entry once it has been deployed.
//   - To change the schema, add a new Vn__ pair and append a Migration below.
//   - Up migrations run inside a transaction; a failure rolls back the whole set.
//   - Down migrations are opt-in and never run automatically by the plugin.
fn db_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "create_initial_schema",
            sql: include_str!("../database/migrations/V1__create_initial_schema/up.sql"),
            kind: MigrationKind::Up,
        },
        // ── Add future migrations here ────────────────────────────────────────
        // Migration {
        //     version: 2,
        //     description: "example_add_column",
        //     sql: include_str!("../database/migrations/V2__example_add_column.up.sql"),
        //     kind: MigrationKind::Up,
        // },
    ]
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:habits.db", db_migrations())
                .build(),
        )
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // A second instance was launched — bring the existing window to focus
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.show();
                let _ = win.set_focus();
            }
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // ── Database ──────────────────────────────────────────────────────
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&app_data_dir)?;
            let db_path = app_data_dir.join("habits.db");
            let conn = Connection::open(&db_path).expect("failed to open SQLite database");

            // Wrap connection in the repository implementation.
            let db = Arc::new(SqliteDb::new(conn));

            // Load persisted settings via the SettingsRepository port.
            let threshold_mins: u64 = db
                .get_setting("idle_threshold_mins")
                .ok()
                .flatten()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5);

            // ── Tracker initial state ─────────────────────────────────────────
            let now = Utc::now();
            let idle_secs = infrastructure::idle::get_idle_seconds();
            let threshold_secs = threshold_mins * 60;

            let initial_status = if idle_secs >= threshold_secs {
                "idle"
            } else {
                "productive"
            };

            // Open the first app session via the SessionRepository port.
            let initial_session_id = db
                .begin_session(&now)
                .expect("failed to create initial session");

            let tracker_state = TrackerState {
                status: initial_status.to_string(),
                session_start: now,
                idle_threshold_secs: threshold_secs,
                current_session_id: initial_session_id,
                current_active_secs: 0,
                current_idle_secs: 0,
                current_locked_secs: 0,
            };

            let tracker = Arc::new(Mutex::new(tracker_state));

            // Clone Arcs so the background task keeps its own references.
            let bg_session_repo = Arc::clone(&db) as Arc<dyn domain::ports::SessionRepository>;
            let bg_app_usage_repo =
                Arc::clone(&db) as Arc<dyn domain::ports::AppUsageRepository>;
            let bg_tracker = Arc::clone(&tracker);

            app.manage(AppState { db, tracker });

            // ── Session lock monitor (WTS notifications) ──────────────────────
            let wake = Arc::new(tokio::sync::Notify::new());
            infrastructure::idle::set_tracker_wake(Arc::clone(&wake));
            infrastructure::idle::start_session_monitor();

            // ── Background tracking task ──────────────────────────────────────
            tauri::async_runtime::spawn(application::tracker::run_tracker(
                bg_session_repo,
                bg_app_usage_repo,
                bg_tracker,
                app.handle().clone(),
                wake,
            ));

            // ── System tray ───────────────────────────────────────────────────
            let show_item =
                MenuItem::with_id(app, "show", "Show Habits", true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &sep, &quit_item])?;

            let initial_tray_icon = if initial_status == "productive" {
                infrastructure::tray_icon::productive_icon()
            } else {
                infrastructure::tray_icon::idle_icon()
            };
            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(initial_tray_icon)
                .tooltip("Habits – Productivity Tracker")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                })
                .build(app)?;

            // ── Intercept window-close → hide to tray ─────────────────────────
            let win = app
                .get_webview_window("main")
                .expect("main window not found");
            win.on_window_event({
                let win = win.clone();
                move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        let _ = win.hide();
                        api.prevent_close();
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            presentation::commands::get_current_status,
            presentation::commands::get_today_stats,
            presentation::commands::get_sessions_for_date,
            presentation::commands::get_history,
            presentation::commands::get_settings,
            presentation::commands::set_settings,
            presentation::commands::get_app_usage,
            presentation::commands::get_app_icon,
            presentation::commands::clear_icon_cache,
            presentation::commands::clear_all_data,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            // RunEvent::Exit fires on every exit path:
            //   • tray Quit → app.exit(0)
            //   • Windows shutdown / logoff (WM_ENDSESSION)
            // RunEvent::Exit fires on every exit path:
            //   • tray Quit → app.exit(0)
            //   • Windows shutdown / logoff (WM_ENDSESSION)
            //   • SIGTERM / SIGBREAK / Ctrl-C
            // Flushing here guarantees the current session always reaches disk.
            if let tauri::RunEvent::Exit = event {
                let now = Utc::now();
                let state = app_handle.state::<AppState>();
                let (session_id, active_secs, idle_secs, locked_secs) = {
                    let t = state.tracker.lock().unwrap();
                    (
                        t.current_session_id,
                        t.current_active_secs,
                        t.current_idle_secs,
                        t.current_locked_secs,
                    )
                };
                let _ = state.db.update_session_time(session_id, active_secs, idle_secs, locked_secs);
                let _ = state.db.end_session(session_id, &now);
            }
        });
}
