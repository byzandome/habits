mod commands;
mod db;
mod idle;
mod tracker;

use std::sync::{Arc, Mutex};

use chrono::Utc;
use rusqlite::Connection;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

// ── Shared application state ──────────────────────────────────────────────────

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub tracker: Arc<Mutex<tracker::TrackerShared>>,
}

// Arc<Mutex<T>> is Send + Sync when T: Send, so Tauri can hold AppState safely.

// ── Entry point ───────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
            db::init_db(&conn).expect("failed to initialise database schema");

            // Load persisted settings
            let threshold_mins: u64 = db::get_setting(&conn, "idle_threshold_mins")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5);

            // ── Tracker initial state ─────────────────────────────────────────
            let now = Utc::now();
            let idle_secs = idle::get_idle_seconds();
            let threshold_secs = threshold_mins * 60;

            let initial_status = if idle_secs >= threshold_secs {
                "idle"
            } else {
                "productive"
            };

            let tracker_shared = tracker::TrackerShared {
                status: initial_status.to_string(),
                session_start: now,
                idle_threshold_secs: threshold_secs,
            };

            let app_state = AppState {
                db: Arc::new(Mutex::new(conn)),
                tracker: Arc::new(Mutex::new(tracker_shared)),
            };

            // Clone Arcs so the bg-task keeps its own references
            let bg_db = Arc::clone(&app_state.db);
            let bg_tracker = Arc::clone(&app_state.tracker);

            app.manage(app_state);

            // ── Background tracking task ──────────────────────────────────────
            tauri::async_runtime::spawn(tracker::run_tracker(bg_db, bg_tracker));

            // ── System tray ───────────────────────────────────────────────────
            let show_item =
                MenuItem::with_id(app, "show", "Show Habits", true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let quit_item =
                MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &sep, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
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
            commands::get_current_status,
            commands::get_today_stats,
            commands::get_sessions_for_date,
            commands::get_history,
            commands::get_settings,
            commands::set_settings,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            // RunEvent::Exit fires on every exit path:
            //   • tray Quit → app.exit(0)
            //   • Windows shutdown / logoff (WM_ENDSESSION)
            //   • SIGTERM / SIGBREAK / Ctrl-C
            // Flushing here guarantees the current segment always reaches disk.
            if let tauri::RunEvent::Exit = event {
                let now = Utc::now();
                let state = app_handle.state::<AppState>();
                let (start, stype) = {
                    let t = state.tracker.lock().unwrap();
                    (t.session_start, t.status.clone())
                };
                if let Ok(conn) = state.db.lock() {
                    let _ = db::insert_session(&conn, &start, &now, &stype);
                };
            }
        });
}
