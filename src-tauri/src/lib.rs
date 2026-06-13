mod db;
mod event;
mod export;
mod google;
mod memo;
mod settings;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, LogicalSize, Manager, PhysicalPosition, PhysicalSize, WindowEvent};

/// Shared application state: a single SQLite connection behind a mutex.
pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    /// True while the window is in the large "expanded" GCal view. While set,
    /// resize events are not persisted so the compact widget size is preserved.
    pub expanded: AtomicBool,
}

fn save_window_setting(handle: &tauri::AppHandle, key: &str, value: i32) {
    let state = handle.state::<AppState>();
    let conn = match state.db.lock() {
        Ok(c) => c,
        Err(_) => return,
    };
    let _ = settings::set(&conn, key, &value.to_string());
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

/// Apply a window display mode to the main widget window.
/// `top` = float above everything, `desktop` = stay at the desktop level
/// (behind other windows), anything else = normal stacking.
fn apply_window_mode(handle: &tauri::AppHandle, mode: &str) {
    if let Some(w) = handle.get_webview_window("main") {
        match mode {
            "top" => {
                let _ = w.set_always_on_bottom(false);
                let _ = w.set_always_on_top(true);
            }
            "desktop" => {
                let _ = w.set_always_on_top(false);
                let _ = w.set_always_on_bottom(true);
            }
            _ => {
                let _ = w.set_always_on_top(false);
                let _ = w.set_always_on_bottom(false);
            }
        }
    }
}

/// Change the window mode and persist the choice in app_settings.
#[tauri::command]
fn set_window_mode(handle: tauri::AppHandle, mode: String) -> Result<(), String> {
    apply_window_mode(&handle, &mode);
    let state = handle.state::<AppState>();
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    settings::set(&conn, "window_mode", &mode).map_err(|e| e.to_string())
}

/// Toggle the window between the compact widget and the large "expanded" GCal
/// view. Expanding grows the window to a fixed large size (resize events are
/// suppressed from persistence); collapsing restores the saved widget size.
#[tauri::command]
fn set_expanded(handle: tauri::AppHandle, expanded: bool) -> Result<(), String> {
    handle.state::<AppState>().expanded.store(expanded, Ordering::SeqCst);
    let Some(w) = handle.get_webview_window("main") else {
        return Ok(());
    };
    if expanded {
        let _ = w.set_size(LogicalSize::new(960.0, 700.0));
    } else {
        // Read the saved compact size, then DROP the db lock before resizing.
        // `set_size` re-enters the window's Resized handler synchronously on this
        // thread; if we still held the (non-reentrant) lock, that handler's own
        // db.lock() would self-deadlock and hang the UI.
        let (cw, ch) = {
            let state = handle.state::<AppState>();
            let conn = state.db.lock().map_err(|e| e.to_string())?;
            let read = |key: &str| -> Option<u32> {
                settings::get(&conn, key).ok().flatten().and_then(|v| v.parse().ok())
            };
            (
                read("win_w").filter(|v| *v > 0).unwrap_or(360),
                read("win_h").filter(|v| *v > 0).unwrap_or(480),
            )
        };
        let _ = w.set_size(PhysicalSize::new(cw, ch));
    }
    Ok(())
}

fn build_tray(app: &tauri::App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "보이기", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "숨기기", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "설정", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "종료", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &settings, &quit])?;

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
            "settings" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                    let _ = w.emit("open-settings", ());
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
                expanded: AtomicBool::new(false),
            });

            // Apply the saved window mode (normal | top | desktop).
            let mode = {
                let state = app.state::<AppState>();
                state
                    .db
                    .lock()
                    .ok()
                    .and_then(|conn| settings::get(&conn, "window_mode").ok().flatten())
                    .unwrap_or_else(|| "normal".to_string())
            };
            apply_window_mode(app.handle(), &mode);

            // Restore the widget window position/size and persist future changes.
            if let Some(window) = app.get_webview_window("main") {
                restore_window(app.handle(), &window);
                let handle = app.handle().clone();
                window.on_window_event(move |event| match event {
                    // Closing the widget hides it to the tray instead of quitting.
                    // Real exit happens via the tray menu's "종료" item.
                    WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();
                        if let Some(w) = handle.get_webview_window("main") {
                            let _ = w.hide();
                        }
                    }
                    WindowEvent::Moved(pos) => {
                        save_window_setting(&handle, "win_x", pos.x);
                        save_window_setting(&handle, "win_y", pos.y);
                    }
                    WindowEvent::Resized(size) => {
                        if !handle.state::<AppState>().expanded.load(Ordering::SeqCst) {
                            save_window_setting(&handle, "win_w", size.width as i32);
                            save_window_setting(&handle, "win_h", size.height as i32);
                        }
                    }
                    _ => {}
                });
            }

            build_tray(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            event::list_calendars,
            event::set_calendar_visible,
            event::list_events_by_range,
            event::create_event,
            event::update_event,
            event::delete_event,
            settings::get_setting,
            settings::set_setting,
            set_window_mode,
            set_expanded,
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
