use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::Emitter;

use crate::config::Rule;
use crate::organizer::{matches_rule_criteria, move_file_with_conflict};

pub struct FolderWatcherState {
    pub is_running: bool,
}

pub struct AppWatcher {
    _watcher: Option<notify::RecommendedWatcher>,
}

impl AppWatcher {
    pub fn new() -> Self {
        Self { _watcher: None }
    }
}

pub fn start_folder_watcher<R: tauri::Runtime>(
    app_handle: tauri::AppHandle<R>,
    rules: Vec<Rule>,
) -> Result<notify::RecommendedWatcher, String> {
    let enabled_rules: Vec<Rule> = rules.into_iter().filter(|r| r.enabled).collect();

    let (tx, rx) = std::sync::mpsc::channel::<Result<Event, notify::Error>>();

    let mut watcher = notify::RecommendedWatcher::new(tx, notify::Config::default())
        .map_err(|e| e.to_string())?;

    for rule in &enabled_rules {
        let folders: Vec<&str> = rule.watch_folder.split(',').map(|w| w.trim()).collect();
        for folder in folders {
            let path = PathBuf::from(folder);
            if path.is_dir() {
                watcher
                    .watch(&path, RecursiveMode::NonRecursive)
                    .ok();
            }
        }
    }

    let rules_arc = Arc::new(Mutex::new(enabled_rules));
    let processing_files: Arc<Mutex<HashSet<PathBuf>>> = Arc::new(Mutex::new(HashSet::new()));

    thread::spawn(move || {
        while let Ok(res) = rx.recv() {
            if let Ok(event) = res {
                if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
                    for path in event.paths {
                        if !path.is_file() {
                            continue;
                        }

                        // Deduplicate concurrent watcher events for the same file
                        {
                            let mut lock = processing_files.lock().unwrap();
                            if lock.contains(&path) {
                                continue;
                            }
                            lock.insert(path.clone());
                        }

                        let rules = rules_arc.lock().unwrap().clone();
                        let app = app_handle.clone();
                        let processing_files_clone = processing_files.clone();
                        let path_clone = path.clone();

                        thread::spawn(move || {
                            thread::sleep(Duration::from_millis(1500));
                            
                            // Cleanup lock after delay regardless of outcome
                            defer_remove(&processing_files_clone, &path_clone);

                            if !path_clone.is_file() {
                                return;
                            }

                            for rule in &rules {
                                if matches_rule_criteria(&path_clone, rule) {
                                    let dest = PathBuf::from(rule.destination.trim());
                                    let filename = path_clone
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("")
                                        .to_string();

                                    match move_file_with_conflict(
                                        &path_clone,
                                        &dest,
                                        &rule.conflict_action,
                                        5,
                                        1000,
                                    ) {
                                        Ok(dest_path) => {
                                            let log_msg = format!(
                                                "[{}] Moved: {} -> \"{}\"",
                                                rule.name,
                                                filename,
                                                dest_path.display()
                                            );
                                            app.emit("log-event", log_msg).ok();
                                        }
                                        Err(err) => {
                                            let log_msg = format!(
                                                "[{}] Info for {}: {}",
                                                rule.name, filename, err
                                            );
                                            app.emit("log-event", log_msg).ok();
                                        }
                                    }
                                    break;
                                }
                            }
                        });
                    }
                }
            }
        }
    });

    Ok(watcher)
}

fn defer_remove(processing_files: &Arc<Mutex<HashSet<PathBuf>>>, path: &PathBuf) {
    let processing_files = processing_files.clone();
    let path = path.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(3));
        let mut lock = processing_files.lock().unwrap();
        lock.remove(&path);
    });
}
