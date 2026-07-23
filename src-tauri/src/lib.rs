pub mod config;
pub mod organizer;
pub mod watcher;

use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, State, WindowEvent,
};
use tauri_plugin_dialog::DialogExt;
use notify::RecommendedWatcher;

use config::{load_config, save_config, AppConfig, Rule};
use organizer::{run_manual_organization, undo_file_move};
use watcher::start_folder_watcher;

pub struct WatcherState {
    pub watcher: Arc<Mutex<Option<RecommendedWatcher>>>,
}

#[tauri::command]
fn get_config_cmd() -> AppConfig {
    load_config()
}

#[tauri::command]
fn save_config_cmd(config: AppConfig, state: State<'_, WatcherState>, app_handle: tauri::AppHandle) -> Result<(), String> {
    save_config(&config)?;

    let mut watcher_guard = state.watcher.lock().unwrap();
    *watcher_guard = None;

    if let Ok(w) = start_folder_watcher(app_handle, config.rules) {
        *watcher_guard = Some(w);
    }

    Ok(())
}

#[tauri::command]
fn run_manual_organize_cmd(rules: Vec<Rule>, app_handle: tauri::AppHandle) -> usize {
    let moved_count = run_manual_organization(&rules);
    let log_msg = format!("Manual organization finished. {} file(s) moved.", moved_count);
    app_handle.emit("log-event", log_msg).ok();
    moved_count
}

#[tauri::command]
fn undo_file_move_cmd(src_path: String, dest_path: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    undo_file_move(&src_path, &dest_path)?;
    let log_msg = format!("Undo successful: Restored {} back to {}", dest_path, src_path);
    app_handle.emit("log-event", log_msg).ok();
    Ok(())
}

#[tauri::command]
async fn pick_folder_cmd(app_handle: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    app_handle.dialog().file().pick_folder(move |folder_path| {
        let _ = tx.send(folder_path.map(|p| p.to_string()));
    });
    rx.recv().map_err(|e| e.to_string())
}

#[tauri::command]
fn restart_watcher_cmd(state: State<'_, WatcherState>, app_handle: tauri::AppHandle) -> Result<(), String> {
    let config = load_config();
    let mut watcher_guard = state.watcher.lock().unwrap();
    *watcher_guard = None;

    if let Ok(w) = start_folder_watcher(app_handle, config.rules) {
        *watcher_guard = Some(w);
    }
    Ok(())
}

#[tauri::command]
fn quit_app_cmd() {
    std::process::exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let initial_config = load_config();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::AppleScript,
            Some(vec!["--flag1", "--flag2"]),
        ))
        .manage(WatcherState {
            watcher: Arc::new(Mutex::new(None)),
        })
        .setup(move |app| {
            let app_handle = app.handle().clone();

            let show_i = MenuItem::with_id(app, "show", "Open Settings", true, None::<&str>)?;
            let organize_i = MenuItem::with_id(app, "organize", "Organize Now", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &organize_i, &quit_i])?;

            let tray = TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().ok();
                            window.set_focus().ok();
                        }
                    }
                    "organize" => {
                        let config = load_config();
                        let count = run_manual_organization(&config.rules);
                        let log_msg = format!("Manual organization from tray finished. {} file(s) moved.", count);
                        app.emit("log-event", log_msg).ok();
                    }
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().ok();
                            window.set_focus().ok();
                        }
                    }
                })
                .build(app)?;

            // Prevent tray icon handle from being dropped when setup function returns
            std::mem::forget(tray);

            let state = app.state::<WatcherState>();
            if let Ok(w) = start_folder_watcher(app_handle, initial_config.rules) {
                *state.watcher.lock().unwrap() = Some(w);
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let config = load_config();
                if config.settings.minimize_to_tray_on_close {
                    api.prevent_close();
                    window.hide().ok();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_config_cmd,
            save_config_cmd,
            run_manual_organize_cmd,
            undo_file_move_cmd,
            pick_folder_cmd,
            restart_watcher_cmd,
            quit_app_cmd
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
