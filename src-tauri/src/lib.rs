mod commands;
mod database;
mod error;
mod file_ops;
mod models;
mod rules;
mod state;
mod watcher;

use std::path::PathBuf;
use std::sync::Arc;

use commands::run_manual_from_tray;
use database::Database;
use file_ops::FileService;
use state::AppState;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, RunEvent, WindowEvent,
};
use watcher::WatcherManager;

fn create_state() -> Result<AppState, error::AppError> {
    let data_root = dirs::data_local_dir()
        .or_else(dirs::data_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("AutoManageFiles");
    std::fs::create_dir_all(&data_root)?;
    let database = Arc::new(Database::open(&data_root.join("auto-manage-files.db"))?);
    database.mark_incomplete_for_review()?;
    let settings = database.settings()?;
    database.prune_activity(settings.history_retention_days)?;
    let files = Arc::new(FileService::new(
        database.clone(),
        data_root.join("recovery"),
    )?);
    Ok(AppState {
        database,
        files,
        watcher: Arc::new(WatcherManager::default()),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = create_state().expect("failed to initialize application state");
    let setup_state = state.clone();

    let application = tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(state)
        .setup(move |app| {
            let show_item =
                MenuItem::with_id(app, "show", "Buka Auto Manage Files", true, None::<&str>)?;
            let organize_item =
                MenuItem::with_id(app, "organize", "Organize sekarang", true, None::<&str>)?;
            let monitoring_item = MenuItem::with_id(
                app,
                "monitoring",
                "Jeda/lanjutkan monitoring",
                true,
                None::<&str>,
            )?;
            let quit_item = MenuItem::with_id(app, "quit", "Keluar", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[&show_item, &organize_item, &monitoring_item, &quit_item],
            )?;

            let tray_state = setup_state.clone();
            let mut tray_builder = TrayIconBuilder::with_id("main-tray")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "show" => show_main_window(app),
                    "organize" => {
                        run_manual_from_tray(app.clone(), tray_state.clone());
                    }
                    "monitoring" => {
                        if let Ok(settings) = tray_state.database.settings() {
                            let enabled = !settings.monitoring_enabled;
                            let _ = tray_state.database.update_settings(models::SettingsPatch {
                                monitoring_enabled: Some(enabled),
                                ..models::SettingsPatch::default()
                            });
                            if enabled {
                                let _ = tray_state.watcher.restart(
                                    app.clone(),
                                    tray_state.database.clone(),
                                    tray_state.files.clone(),
                                );
                            } else {
                                tray_state.watcher.pause(app);
                            }
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
                        show_main_window(tray.app_handle());
                    }
                });
            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }
            tray_builder.build(app)?;

            let settings = setup_state.database.settings()?;
            if settings.monitoring_enabled {
                setup_state.watcher.restart(
                    app.handle().clone(),
                    setup_state.database.clone(),
                    setup_state.files.clone(),
                )?;
            } else {
                setup_state.watcher.pause(app.handle());
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<AppState>();
                if state
                    .database
                    .settings()
                    .map(|settings| settings.minimize_to_tray_on_close)
                    .unwrap_or(true)
                {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_bootstrap,
            commands::list_rules,
            commands::upsert_rule,
            commands::delete_rule,
            commands::set_rule_enabled,
            commands::reorder_rules,
            commands::preview_rules,
            commands::execute_plan,
            commands::list_activity,
            commands::execute_undo,
            commands::set_monitoring,
            commands::update_settings,
            commands::pick_folder,
            commands::export_config,
            commands::import_config,
            commands::shutdown_app,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Tauri application");

    application.run(|app, event| {
        if matches!(event, RunEvent::ExitRequested { .. } | RunEvent::Exit) {
            app.state::<AppState>().watcher.shutdown();
        }
    });
}

fn show_main_window<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
