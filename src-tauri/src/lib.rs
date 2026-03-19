mod application;
mod domain;
mod infrastructure;
mod presentation;
mod schema;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::Connection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

use domain::entities::TrackerState;
use domain::ports::SettingsRepository;
use infrastructure::db::SqliteDb;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

// ── Shared application state ──────────────────────────────────────────────────

pub struct AppState {
    pub db: Arc<SqliteDb>,
    pub tracker: Arc<Mutex<TrackerState>>,
    pub icons_dir: PathBuf,
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
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
            let _ = dotenvy::dotenv();

            let db_url = match std::env::var("DATABASE_URL") {
                Ok(url) => url,
                Err(_) => {
                    let dir = app
                        .path()
                        .app_data_dir()
                        .expect("failed to resolve app data dir");
                    std::fs::create_dir_all(&dir)?;
                    format!(
                        "sqlite://{}",
                        dir.join("habits.db").to_string_lossy().replace('\\', "/")
                    )
                }
            };

            let mut conn =
                SqliteConnection::establish(&db_url).expect("failed to open SQLite database");

            // Enable foreign key enforcement for this connection.
            diesel::sql_query("PRAGMA foreign_keys = ON")
                .execute(&mut conn)
                .expect("failed to enable foreign keys");

            conn.run_pending_migrations(MIGRATIONS)
                .expect("failed to run database migrations");

            let db = Arc::new(SqliteDb::new(conn));

            // ── Icons cache directory ─────────────────────────────────────────
            let icons_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir")
                .join("icons");
            std::fs::create_dir_all(&icons_dir)?;

            // ── Tracker initial state ─────────────────────────────────────────
            let threshold_mins: u64 = db
                .get_setting("idle_threshold_mins")
                .ok()
                .flatten()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5);

            let now = Utc::now();
            let idle_secs = infrastructure::idle::get_idle_seconds();
            let threshold_secs = threshold_mins * 60;

            let initial_status = if idle_secs >= threshold_secs { "idle" } else { "productive" };

            let tracker = Arc::new(Mutex::new(TrackerState {
                status: initial_status.to_string(),
                session_start: now,
                idle_threshold_secs: threshold_secs,
                current_app_usage_id: None,
                current_app_path: None,
                current_usage_start: None,
            }));

            let bg_app_repo =
                Arc::clone(&db) as Arc<dyn domain::ports::AppRepository>;
            let bg_usage_repo =
                Arc::clone(&db) as Arc<dyn domain::ports::AppUsageRepository>;
            let bg_tracker = Arc::clone(&tracker);

            app.manage(AppState { db, tracker, icons_dir });

            // ── Session lock monitor ──────────────────────────────────────────
            let wake = Arc::new(tokio::sync::Notify::new());
            infrastructure::idle::set_tracker_wake(Arc::clone(&wake));
            infrastructure::idle::start_session_monitor();

            // ── Background tracking task ──────────────────────────────────────
            tauri::async_runtime::spawn(application::tracker::run_tracker(
                bg_app_repo,
                bg_usage_repo,
                bg_tracker,
                app.handle().clone(),
                wake,
            ));

            // ── System tray ───────────────────────────────────────────────────
            let show_item = MenuItem::with_id(app, "show", "Show Habits", true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &sep, &quit_item])?;

            let initial_icon = if initial_status == "productive" {
                infrastructure::tray_icon::productive_icon()
            } else {
                infrastructure::tray_icon::idle_icon()
            };
            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(initial_icon)
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
            let win = app.get_webview_window("main").expect("main window not found");
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
            presentation::commands::get_settings,
            presentation::commands::set_settings,
            presentation::commands::get_apps,
            presentation::commands::get_app_usages,
            presentation::commands::get_domains,
            presentation::commands::get_domain_history,
            presentation::commands::get_app_icon,
            presentation::commands::clear_icon_cache,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                let now = Utc::now();
                let state = app_handle.state::<AppState>();
                let (usage_id, usage_start) = {
                    let t = state.tracker.lock().unwrap();
                    (t.current_app_usage_id.clone(), t.current_usage_start)
                };
                if let (Some(uid), Some(start)) = (usage_id, usage_start) {
                    use chrono::SecondsFormat;
                    let dur = (now - start).num_seconds().max(0);
                    let end_str = now.to_rfc3339_opts(SecondsFormat::Secs, true);
                    use domain::ports::AppUsageRepository;
                    let _ = state.db.end_usage(&uid, &end_str, dur);
                }
            }
        });
}