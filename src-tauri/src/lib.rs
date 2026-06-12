mod db;
mod export;
mod google;
mod memo;
mod settings;

use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{
    Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder, WindowEvent,
};

/// Shared application state: a single SQLite connection behind a mutex.
pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
}

/// Per-view date handed to child windows (memo/settings) at open time,
/// fetched by the window on mount via `get_view_date`.
pub struct ViewState {
    pub dates: Mutex<std::collections::HashMap<String, String>>,
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

/// Open (or focus, if already open) a child window loading the SPA with a
/// `?view=` query so the frontend mounts the right root component.
fn open_child_window(
    app: &tauri::AppHandle,
    label: &str,
    url: String,
    title: &str,
    width: f64,
    height: f64,
) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(label) {
        let _ = win.show();
        let _ = win.set_focus();
        return Ok(());
    }
    WebviewWindowBuilder::new(app, label, WebviewUrl::App(url.into()))
        .title(title)
        .inner_size(width, height)
        .min_inner_size(300.0, 360.0)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Record the date a child window should display, then (re)open it.
fn remember_view_date(app: &tauri::AppHandle, view: &str, date: &str) {
    if let Ok(mut m) = app.state::<ViewState>().dates.lock() {
        m.insert(view.to_string(), date.to_string());
    }
}

/// Open the memo editor window for a given date (reused across dates).
#[tauri::command]
fn open_memo_window(app: tauri::AppHandle, date: String) -> Result<(), String> {
    remember_view_date(&app, "memo", &date);
    let existed = app.get_webview_window("memo").is_some();
    open_child_window(&app, "memo", "index.html".into(), "메모", 340.0, 460.0)?;
    // If it was already open, tell it which date to show now.
    if existed {
        if let Some(win) = app.get_webview_window("memo") {
            let _ = win.emit("memo-date", date);
        }
    }
    Ok(())
}

/// Open the settings window.
#[tauri::command]
fn open_settings_window(app: tauri::AppHandle, date: String) -> Result<(), String> {
    remember_view_date(&app, "settings", &date);
    let existed = app.get_webview_window("settings").is_some();
    open_child_window(&app, "settings", "index.html".into(), "설정", 360.0, 540.0)?;
    // If it was already open, update date-dependent settings such as export scope.
    if existed {
        if let Some(win) = app.get_webview_window("settings") {
            let _ = win.emit("settings-date", date);
        }
    }
    Ok(())
}

/// The date a child window should display (set when the window was opened).
#[tauri::command]
fn get_view_date(app: tauri::AppHandle, view: String) -> String {
    app.state::<ViewState>()
        .dates
        .lock()
        .ok()
        .and_then(|m| m.get(&view).cloned())
        .unwrap_or_default()
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
                let today = chrono::Local::now().format("%Y-%m-%d").to_string();
                let _ = open_settings_window(app.clone(), today);
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
            app.manage(ViewState {
                dates: Mutex::new(std::collections::HashMap::new()),
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
            memo::list_memos_by_month,
            memo::list_memos_by_date,
            memo::create_memo,
            memo::update_memo,
            memo::delete_memo,
            settings::get_setting,
            settings::set_setting,
            set_window_mode,
            open_memo_window,
            open_settings_window,
            get_view_date,
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
