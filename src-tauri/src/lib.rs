mod db;
mod export;
mod google;
mod memo;
mod settings;

use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, PhysicalPosition, PhysicalSize, WindowEvent};

/// Shared application state: a single SQLite connection behind a mutex.
pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
}

fn save_window_setting(handle: &tauri::AppHandle, key: &str, value: i32) {
    let state = handle.state::<AppState>();
    if let Ok(conn) = state.db.lock() {
        let _ = settings::set(&conn, key, &value.to_string());
    }
}

fn restore_window(handle: &tauri::AppHandle, window: &tauri::WebviewWindow) {
    let state = handle.state::<AppState>();
    let conn = match state.db.lock() {
        Ok(c) => c,
        Err(_) => return,
    };
    let read = |key: &str| -> Option<i32> {
        settings::get(&conn, key).ok().flatten().and_then(|v| v.parse().ok())
    };
    if let (Some(x), Some(y)) = (read("win_x"), read("win_y")) {
        let _ = window.set_position(PhysicalPosition::new(x, y));
    }
    if let (Some(w), Some(h)) = (read("win_w"), read("win_h")) {
        if w > 0 && h > 0 {
            let _ = window.set_size(PhysicalSize::new(w as u32, h as u32));
        }
    }
}

fn build_tray(app: &tauri::App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "보이기", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "숨기기", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "종료", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &quit])?;

    TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("duckCalendar")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            "hide" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.hide();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        })
        .build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Open the local database in the app data directory.
            let dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&dir).ok();
            let conn = db::open(dir.join("duckCalendar.db")).expect("failed to open database");
            app.manage(AppState {
                db: Mutex::new(conn),
            });

            // Restore the widget window position/size and persist future changes.
            if let Some(window) = app.get_webview_window("main") {
                restore_window(app.handle(), &window);
                let handle = app.handle().clone();
                window.on_window_event(move |event| match event {
                    WindowEvent::Moved(pos) => {
                        save_window_setting(&handle, "win_x", pos.x);
                        save_window_setting(&handle, "win_y", pos.y);
                    }
                    WindowEvent::Resized(size) => {
                        save_window_setting(&handle, "win_w", size.width as i32);
                        save_window_setting(&handle, "win_h", size.height as i32);
                    }
                    _ => {}
                });
            }

            build_tray(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            memo::list_memo_dates,
            memo::list_memos_by_date,
            memo::create_memo,
            memo::update_memo,
            memo::delete_memo,
            settings::get_setting,
            settings::set_setting,
            export::export_data,
            export::export_to_file,
            google::oauth::google_connect,
            google::oauth::google_disconnect,
            google::oauth::google_status,
            google::sync::sync_memo,
            google::sync::sync_selected,
            google::sync::sync_status_map,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
