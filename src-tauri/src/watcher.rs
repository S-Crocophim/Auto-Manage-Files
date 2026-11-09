use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, UNIX_EPOCH};

use crossbeam_channel::{bounded, select, Receiver, Sender, TrySendError};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::database::Database;
use crate::error::{AppError, AppResult};
use crate::file_ops::FileService;
use crate::models::MonitoringHealthDto;
use crate::rules::{build_operation, matching_rule};

const QUEUE_CAPACITY: usize = 512;
const DEDUPE_WINDOW: Duration = Duration::from_secs(3);

enum WorkItem {
    Path(PathBuf),
    Error(String),
}

struct WatcherRuntime {
    watcher: RecommendedWatcher,
    stop_tx: Sender<()>,
    worker: Option<JoinHandle<()>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AttentionEvent {
    code: String,
    message: String,
    rule_id: Option<String>,
    operation_id: Option<String>,
}

pub struct WatcherManager {
    runtime: Mutex<Option<WatcherRuntime>>,
    health: Arc<Mutex<MonitoringHealthDto>>,
    queue_depth: Arc<AtomicUsize>,
    active_jobs: Arc<AtomicUsize>,
}

impl Default for WatcherManager {
    fn default() -> Self {
        Self {
            runtime: Mutex::new(None),
            health: Arc::new(Mutex::new(MonitoringHealthDto::default())),
            queue_depth: Arc::new(AtomicUsize::new(0)),
            active_jobs: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl WatcherManager {
    pub fn health(&self) -> MonitoringHealthDto {
        let mut value = self
            .health
            .lock()
            .map(|health| health.clone())
            .unwrap_or_else(|_| MonitoringHealthDto {
                state: "error".into(),
                last_error: Some("Monitoring health lock is unavailable".into()),
                ..MonitoringHealthDto::default()
            });
        value.queue_depth = self.queue_depth.load(Ordering::Relaxed);
        value.active_jobs = self.active_jobs.load(Ordering::Relaxed);
        value
    }

    pub fn restart(
        &self,
        app: AppHandle,
        database: Arc<Database>,
        files: Arc<FileService>,
    ) -> AppResult<MonitoringHealthDto> {
        self.stop_runtime("stopped");
        let settings = database.settings()?;
        if !settings.monitoring_enabled {
            self.update_health("paused", 0, None);
            emit_health(&app, &self.health());
            return Ok(self.health());
        }

        self.update_health("starting", 0, None);
        emit_health(&app, &self.health());
        let rules = database.list_rules()?;
        let mut watched: HashMap<PathBuf, bool> = HashMap::new();
        for rule in rules.iter().filter(|rule| rule.enabled) {
            for source in &rule.sources {
                let path = PathBuf::from(&source.canonical_path);
                if path.is_dir() {
                    watched
                        .entry(path)
                        .and_modify(|recursive| *recursive |= rule.recursive)
                        .or_insert(rule.recursive);
                }
            }
        }

        let (work_tx, work_rx) = bounded::<WorkItem>(QUEUE_CAPACITY);
        let queue_depth = self.queue_depth.clone();
        let callback_active = self.active_jobs.clone();
        let health = self.health.clone();
        let callback_app = app.clone();
        let mut watcher = RecommendedWatcher::new(
            move |result: Result<Event, notify::Error>| match result {
                Ok(event)
                    if matches!(
                        event.kind,
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Any
                    ) =>
                {
                    for path in event.paths {
                        queue_depth.fetch_add(1, Ordering::Relaxed);
                        match work_tx.try_send(WorkItem::Path(path)) {
                            Ok(()) => {}
                            Err(TrySendError::Full(_)) => {
                                decrement(&queue_depth);
                                set_health_error(
                                    &health,
                                    "Monitoring queue is full; a filesystem event was skipped",
                                );
                                emit_health_from_parts(
                                    &callback_app,
                                    &health,
                                    &queue_depth,
                                    &callback_active,
                                );
                            }
                            Err(TrySendError::Disconnected(_)) => {
                                decrement(&queue_depth);
                            }
                        }
                    }
                }
                Ok(_) => {}
                Err(error) => {
                    let _ = work_tx.try_send(WorkItem::Error(error.to_string()));
                }
            },
            notify::Config::default(),
        )?;

        let mut watched_count = 0;
        let mut watch_warnings = Vec::new();
        for (path, recursive) in watched {
            let mode = if recursive {
                RecursiveMode::Recursive
            } else {
                RecursiveMode::NonRecursive
            };
            match watcher.watch(&path, mode) {
                Ok(()) => watched_count += 1,
                Err(error) => watch_warnings.push(format!("{}: {error}", path.display())),
            }
        }

        if watched_count == 0 {
            let error = if watch_warnings.is_empty() {
                "No enabled rule has an accessible source folder".to_string()
            } else {
                watch_warnings.join("; ")
            };
            self.update_health("attention", 0, Some(error));
        } else {
            self.update_health(
                if watch_warnings.is_empty() {
                    "monitoring"
                } else {
                    "attention"
                },
                watched_count,
                (!watch_warnings.is_empty()).then(|| watch_warnings.join("; ")),
            );
        }

        let (stop_tx, stop_rx) = bounded::<()>(1);
        let worker_app = app.clone();
        let worker_health = self.health.clone();
        let worker_queue = self.queue_depth.clone();
        let worker_active = self.active_jobs.clone();
        let worker = thread::Builder::new()
            .name("amf-watcher-worker".into())
            .spawn(move || {
                worker_loop(
                    worker_app,
                    database,
                    files,
                    work_rx,
                    stop_rx,
                    worker_health,
                    worker_queue,
                    worker_active,
                )
            })
            .map_err(|error| AppError::new("WATCHER_THREAD_ERROR", error.to_string()))?;

        let mut runtime = self
            .runtime
            .lock()
            .map_err(|_| AppError::new("WATCHER_LOCK_POISONED", "Watcher lock is unavailable"))?;
        *runtime = Some(WatcherRuntime {
            watcher,
            stop_tx,
            worker: Some(worker),
        });
        drop(runtime);
        let snapshot = self.health();
        emit_health(&app, &snapshot);
        Ok(snapshot)
    }

    pub fn pause(&self, app: &AppHandle) -> MonitoringHealthDto {
        self.stop_runtime("paused");
        let snapshot = self.health();
        emit_health(app, &snapshot);
        snapshot
    }

    pub fn shutdown(&self) {
        self.stop_runtime("stopped");
    }

    fn stop_runtime(&self, final_state: &str) {
        let runtime = self.runtime.lock().ok().and_then(|mut value| value.take());
        if let Some(mut runtime) = runtime {
            let _ = runtime.stop_tx.try_send(());
            drop(runtime.stop_tx);
            drop(runtime.watcher);
            if let Some(worker) = runtime.worker.take() {
                let _ = worker.join();
            }
        }
        self.queue_depth.store(0, Ordering::Relaxed);
        self.active_jobs.store(0, Ordering::Relaxed);
        let watched_source_count = self
            .health
            .lock()
            .map(|value| value.watched_source_count)
            .unwrap_or_default();
        self.update_health(final_state, watched_source_count, None);
    }

    fn update_health(&self, state: &str, watched_source_count: usize, last_error: Option<String>) {
        if let Ok(mut health) = self.health.lock() {
            health.state = state.into();
            health.watched_source_count = watched_source_count;
            health.last_error = last_error;
            health.queue_depth = self.queue_depth.load(Ordering::Relaxed);
            health.active_jobs = self.active_jobs.load(Ordering::Relaxed);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn worker_loop(
    app: AppHandle,
    database: Arc<Database>,
    files: Arc<FileService>,
    work_rx: Receiver<WorkItem>,
    stop_rx: Receiver<()>,
    health: Arc<Mutex<MonitoringHealthDto>>,
    queue_depth: Arc<AtomicUsize>,
    active_jobs: Arc<AtomicUsize>,
) {
    let mut recently_seen: HashMap<PathBuf, Instant> = HashMap::new();
    loop {
        select! {
            recv(stop_rx) -> _ => break,
            recv(work_rx) -> message => {
                let Ok(message) = message else { break };
                decrement(&queue_depth);
                match message {
                    WorkItem::Error(error) => {
                        set_health_error(&health, &error);
                        emit_attention(&app, "WATCHER_ERROR", &error, None, None);
                    }
                    WorkItem::Path(path) => {
                        let now = Instant::now();
                        recently_seen.retain(|_, seen| now.duration_since(*seen) < DEDUPE_WINDOW);
                        if recently_seen.get(&path).is_some_and(|seen| now.duration_since(*seen) < DEDUPE_WINDOW) {
                            continue;
                        }
                        recently_seen.insert(path.clone(), now);
                        let settings = match database.settings() {
                            Ok(value) => value,
                            Err(error) => {
                                set_health_error(&health, &error.message);
                                continue;
                            }
                        };
                        if !wait_until_stable(&path, settings.stability_window_ms, &stop_rx) {
                            continue;
                        }
                        let rules = match database.list_rules() {
                            Ok(value) => value,
                            Err(error) => {
                                set_health_error(&health, &error.message);
                                continue;
                            }
                        };
                        let Some(rule) = matching_rule(&path, &rules) else {
                            continue;
                        };
                        let batch_id = uuid::Uuid::new_v4().to_string();
                        let item = match build_operation(rule, &path, &batch_id, "realtime", "pending") {
                            Ok(value) => value,
                            Err(error) => {
                                emit_attention(&app, &error.code, &error.message, Some(&rule.id), None);
                                continue;
                            }
                        };
                        match database.insert_realtime_item(&item) {
                            Ok(true) => {}
                            Ok(false) => continue,
                            Err(error) => {
                                emit_attention(&app, &error.code, &error.message, Some(&rule.id), Some(&item.id));
                                continue;
                            }
                        }

                        active_jobs.fetch_add(1, Ordering::Relaxed);
                        emit_health_from_parts(&app, &health, &queue_depth, &active_jobs);
                        let operation_id = item.id.clone();
                        let result = files.execute_operation(item);
                        decrement(&active_jobs);
                        let persisted = database.get_operation(&operation_id).ok();
                        if let Some(operation) = persisted.as_ref() {
                            let _ = app.emit("amf://activity", operation.activity());
                        }
                        if let Ok(summary) = database.finish_batch(&batch_id) {
                            let _ = app.emit("amf://batch-progress", summary);
                        }
                        if let Err(error) = result {
                            emit_attention(
                                &app,
                                &error.code,
                                &error.message,
                                Some(&rule.id),
                                Some(&operation_id),
                            );
                        }
                        emit_health_from_parts(&app, &health, &queue_depth, &active_jobs);
                    }
                }
            }
        }
    }
}

fn wait_until_stable(path: &Path, window_ms: u64, stop_rx: &Receiver<()>) -> bool {
    let delay = Duration::from_millis(window_ms.clamp(250, 30_000));
    let mut previous = fingerprint(path);
    if previous.is_none() {
        return false;
    }
    for _ in 0..4 {
        select! {
            recv(stop_rx) -> _ => return false,
            recv(crossbeam_channel::after(delay)) -> _ => {}
        }
        let current = fingerprint(path);
        if current.is_some() && current == previous {
            return true;
        }
        previous = current;
        if previous.is_none() {
            return false;
        }
    }
    false
}

fn fingerprint(path: &Path) -> Option<(u64, u128)> {
    let metadata = path.metadata().ok()?;
    if !metadata.is_file() {
        return None;
    }
    let modified = metadata
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_millis();
    Some((metadata.len(), modified))
}

fn set_health_error(health: &Arc<Mutex<MonitoringHealthDto>>, message: &str) {
    if let Ok(mut health) = health.lock() {
        health.state = "attention".into();
        health.last_error = Some(message.into());
    }
}

fn emit_health(app: &AppHandle, value: &MonitoringHealthDto) {
    let _ = app.emit("amf://monitoring-health", value);
}

fn emit_health_from_parts(
    app: &AppHandle,
    health: &Arc<Mutex<MonitoringHealthDto>>,
    queue_depth: &AtomicUsize,
    active_jobs: &AtomicUsize,
) {
    if let Ok(mut value) = health.lock() {
        value.queue_depth = queue_depth.load(Ordering::Relaxed);
        value.active_jobs = active_jobs.load(Ordering::Relaxed);
        emit_health(app, &value);
    }
}

fn emit_attention(
    app: &AppHandle,
    code: &str,
    message: &str,
    rule_id: Option<&str>,
    operation_id: Option<&str>,
) {
    let _ = app.emit(
        "amf://attention",
        AttentionEvent {
            code: code.into(),
            message: message.into(),
            rule_id: rule_id.map(str::to_string),
            operation_id: operation_id.map(str::to_string),
        },
    );
}

fn decrement(value: &AtomicUsize) {
    let _ = value.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
        Some(current.saturating_sub(1))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_changes_when_file_changes() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("file.txt");
        std::fs::write(&path, b"a").expect("first write");
        let before = fingerprint(&path).expect("first fingerprint");
        std::fs::write(&path, b"longer").expect("second write");
        let after = fingerprint(&path).expect("second fingerprint");
        assert_ne!(before, after);
    }
}
