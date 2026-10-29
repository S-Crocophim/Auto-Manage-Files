use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Emitter, State};
use tauri_plugin_autostart::ManagerExt as AutostartManagerExt;
use tauri_plugin_dialog::DialogExt;

use crate::error::{AppError, AppResult};
use crate::models::{
    now_rfc3339, ActivityDto, ActivityPageDto, ActivityQuery, BatchSummaryDto, BootstrapDto,
    ConfigExport, ConfigPathRequest, DeleteRuleRequest, ExecutePlanRequest, ExecuteUndoRequest,
    MonitoringHealthDto, PreviewItemDto, PreviewResponseDto, PreviewRulesRequest,
    ReorderRulesRequest, RuleDto, SetMonitoringRequest, SetRuleEnabledRequest, SettingsDto,
    SettingsPatch, UpdateSettingsRequest, UpsertRuleRequest,
};
use crate::rules::{
    build_operation, normalize_imported_rule, normalize_rule, scan_matching_files,
    validate_rule_graph,
};
use crate::state::AppState;

const MAX_CONFIG_BYTES: u64 = 5 * 1024 * 1024;

#[tauri::command]
pub fn get_bootstrap(state: State<'_, AppState>) -> AppResult<BootstrapDto> {
    let activity = state.database.list_activity(ActivityQuery {
        limit: Some(20),
        ..ActivityQuery::default()
    })?;
    Ok(BootstrapDto {
        rules: state.database.list_rules()?,
        settings: state.database.settings()?,
        monitoring: state.watcher.health(),
        recent_activity: activity.items,
    })
}

#[tauri::command]
pub fn list_rules(state: State<'_, AppState>) -> AppResult<Vec<RuleDto>> {
    state.database.list_rules()
}

#[tauri::command]
pub fn upsert_rule(
    app: AppHandle,
    state: State<'_, AppState>,
    request: UpsertRuleRequest,
) -> AppResult<RuleDto> {
    let existing = request
        .id
        .as_deref()
        .map(|id| state.database.get_rule(id))
        .transpose()?
        .flatten();
    let rule = normalize_rule(request, existing.as_ref(), state.database.next_priority()?)?;
    let mut rules = state.database.list_rules()?;
    if let Some(position) = rules.iter().position(|item| item.id == rule.id) {
        rules[position] = rule.clone();
    } else {
        rules.push(rule.clone());
    }
    validate_rule_graph(&rules)?;
    state.database.save_rule(&rule)?;
    refresh_monitoring(&app, &state)?;
    state
        .database
        .get_rule(&rule.id)?
        .ok_or_else(|| AppError::new("RULE_NOT_FOUND", "Saved rule could not be loaded"))
}

#[tauri::command]
pub fn delete_rule(
    app: AppHandle,
    state: State<'_, AppState>,
    request: DeleteRuleRequest,
) -> AppResult<()> {
    state.database.delete_rule(&request.id)?;
    refresh_monitoring(&app, &state)
}

#[tauri::command]
pub fn set_rule_enabled(
    app: AppHandle,
    state: State<'_, AppState>,
    request: SetRuleEnabledRequest,
) -> AppResult<RuleDto> {
    let mut rules = state.database.list_rules()?;
    let rule = rules
        .iter_mut()
        .find(|rule| rule.id == request.id)
        .ok_or_else(|| AppError::new("RULE_NOT_FOUND", "Rule was not found"))?;
    rule.enabled = request.enabled;
    validate_rule_graph(&rules)?;
    state
        .database
        .set_rule_enabled(&request.id, request.enabled)?;
    refresh_monitoring(&app, &state)?;
    state
        .database
        .get_rule(&request.id)?
        .ok_or_else(|| AppError::new("RULE_NOT_FOUND", "Updated rule could not be loaded"))
}

#[tauri::command]
pub fn reorder_rules(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ReorderRulesRequest,
) -> AppResult<Vec<RuleDto>> {
    state.database.reorder_rules(&request.ordered_ids)?;
    refresh_monitoring(&app, &state)?;
    state.database.list_rules()
}

#[tauri::command]
pub async fn preview_rules(
    state: State<'_, AppState>,
    request: PreviewRulesRequest,
) -> AppResult<PreviewResponseDto> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || preview_impl(&state, request))
        .await
        .map_err(|error| AppError::new("PREVIEW_TASK_FAILED", error.to_string()))?
}

#[tauri::command]
pub async fn execute_plan(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ExecutePlanRequest,
) -> AppResult<BatchSummaryDto> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || execute_plan_impl(&app, &state, request))
        .await
        .map_err(|error| AppError::new("EXECUTION_TASK_FAILED", error.to_string()))?
}

#[tauri::command]
pub fn list_activity(
    state: State<'_, AppState>,
    request: ActivityQuery,
) -> AppResult<ActivityPageDto> {
    state.database.list_activity(request)
}

#[tauri::command]
pub async fn execute_undo(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ExecuteUndoRequest,
) -> AppResult<ActivityDto> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let activity = state
            .files
            .execute_undo(&request.operation_id, request.conflict_policy)?;
        let _ = app.emit("amf://activity", &activity);
        Ok(activity)
    })
    .await
    .map_err(|error| AppError::new("UNDO_TASK_FAILED", error.to_string()))?
}

#[tauri::command]
pub fn set_monitoring(
    app: AppHandle,
    state: State<'_, AppState>,
    request: SetMonitoringRequest,
) -> AppResult<MonitoringHealthDto> {
    state.database.update_settings(SettingsPatch {
        monitoring_enabled: Some(request.enabled),
        ..SettingsPatch::default()
    })?;
    if request.enabled {
        state
            .watcher
            .restart(app, state.database.clone(), state.files.clone())
    } else {
        Ok(state.watcher.pause(&app))
    }
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    request: UpdateSettingsRequest,
) -> AppResult<SettingsDto> {
    let previous = state.database.settings()?;
    let settings = state.database.update_settings(request.settings)?;
    if settings.auto_start != previous.auto_start {
        if let Err(error) = apply_autostart(&app, settings.auto_start) {
            let _ = state.database.update_settings(SettingsPatch {
                auto_start: Some(previous.auto_start),
                ..SettingsPatch::default()
            });
            return Err(error);
        }
    }
    if settings.monitoring_enabled != previous.monitoring_enabled
        || settings.stability_window_ms != previous.stability_window_ms
    {
        if settings.monitoring_enabled {
            state
                .watcher
                .restart(app.clone(), state.database.clone(), state.files.clone())?;
        } else {
            state.watcher.pause(&app);
        }
    }
    state
        .database
        .prune_activity(settings.history_retention_days)?;
    Ok(settings)
}

#[tauri::command]
pub async fn pick_folder(app: AppHandle) -> AppResult<Option<String>> {
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog().file().pick_folder(move |path| {
        let _ = tx.send(path.map(|value| value.to_string()));
    });
    rx.recv()
        .map_err(|error| AppError::new("DIALOG_CLOSED", error.to_string()))
}

#[tauri::command]
pub async fn export_config(
    state: State<'_, AppState>,
    request: ConfigPathRequest,
) -> AppResult<()> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let path = validated_config_path(&request.path)?;
        let export = ConfigExport {
            schema_version: 2,
            exported_at: now_rfc3339(),
            rules: state.database.list_rules()?,
            settings: state.database.settings()?,
        };
        let bytes = serde_json::to_vec_pretty(&export)
            .map_err(|error| AppError::new("CONFIG_SERIALIZE_ERROR", error.to_string()))?;
        atomic_replace(&path, &bytes)
    })
    .await
    .map_err(|error| AppError::new("EXPORT_TASK_FAILED", error.to_string()))?
}

#[tauri::command]
pub async fn import_config(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ConfigPathRequest,
) -> AppResult<BootstrapDto> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let path = validated_config_path(&request.path)?;
        let metadata = path.metadata().map_err(|error| {
            AppError::new(
                "CONFIG_READ_ERROR",
                format!("Cannot read configuration file: {error}"),
            )
        })?;
        if metadata.len() > MAX_CONFIG_BYTES {
            return Err(AppError::new(
                "CONFIG_TOO_LARGE",
                "Configuration file cannot exceed 5 MiB",
            ));
        }
        let bytes = fs::read(&path)?;
        let export: ConfigExport = serde_json::from_slice(&bytes)
            .map_err(|error| AppError::new("CONFIG_PARSE_ERROR", error.to_string()))?;
        if export.schema_version != 2 {
            return Err(AppError::new(
                "UNSUPPORTED_CONFIG_VERSION",
                "Only configuration schema version 2 is supported",
            ));
        }

        let mut rules = Vec::with_capacity(export.rules.len());
        let mut ids = HashSet::new();
        for (priority, rule) in export.rules.into_iter().enumerate() {
            let rule = normalize_imported_rule(rule, priority as i64)?;
            if !ids.insert(rule.id.clone()) {
                return Err(AppError::new(
                    "DUPLICATE_RULE_ID",
                    "Imported configuration contains duplicate rule ids",
                ));
            }
            rules.push(rule);
        }
        validate_rule_graph(&rules)?;

        let previous = state.database.settings()?;
        state.database.replace_config(&rules, &export.settings)?;
        if let Err(error) = apply_autostart(&app, export.settings.auto_start) {
            let _ = state.database.update_settings(SettingsPatch {
                auto_start: Some(previous.auto_start),
                ..SettingsPatch::default()
            });
            return Err(error);
        }
        if export.settings.monitoring_enabled {
            state
                .watcher
                .restart(app.clone(), state.database.clone(), state.files.clone())?;
        } else {
            state.watcher.pause(&app);
        }
        get_bootstrap_from_state(&state)
    })
    .await
    .map_err(|error| AppError::new("IMPORT_TASK_FAILED", error.to_string()))?
}

#[tauri::command]
pub fn shutdown_app(app: AppHandle, state: State<'_, AppState>) {
    state.watcher.shutdown();
    app.exit(0);
}

pub fn preview_impl(
    state: &AppState,
    request: PreviewRulesRequest,
) -> AppResult<PreviewResponseDto> {
    let requested_ids: HashSet<&str> = request.rule_ids.iter().map(String::as_str).collect();
    let rules = state
        .database
        .list_rules()?
        .into_iter()
        .filter(|rule| {
            (requested_ids.is_empty() && rule.enabled)
                || requested_ids.contains(rule.id.as_str())
        })
        .collect::<Vec<_>>();
    if !requested_ids.is_empty()
        && !requested_ids
            .iter()
            .all(|id| rules.iter().any(|rule| rule.id == *id))
    {
        return Err(AppError::new(
            "RULE_NOT_FOUND",
            "One or more requested rules are unavailable",
        ));
    }

    let limit = request.limit.unwrap_or(500).clamp(1, 5_000);
    let plan_id = uuid::Uuid::new_v4().to_string();
    let mut records = Vec::new();
    let mut preview_items = Vec::new();
    let mut warnings = Vec::new();
    let mut claimed = HashSet::new();
    let mut total_count = 0;
    for rule in &rules {
        let remaining = limit.saturating_sub(records.len());
        let (paths, count, _, rule_warnings) = scan_matching_files(rule, remaining, &mut claimed);
        total_count += count;
        warnings.extend(rule_warnings);
        for path in paths {
            let record = build_operation(rule, &path, &plan_id, "manual", "planned")?;
            let destination_exists = Path::new(&record.destination_path).exists();
            let status = match (record.conflict_policy, destination_exists) {
                (crate::models::ConflictPolicy::Skip, true) => "willSkip",
                (crate::models::ConflictPolicy::SafeOverwrite, true) => "willOverwrite",
                _ => "ready",
            };
            preview_items.push(PreviewItemDto {
                id: record.id.clone(),
                rule_id: rule.id.clone(),
                rule_name: rule.name.clone(),
                source_path: record.source_path.clone(),
                destination_path: record.destination_path.clone(),
                size_bytes: record.source_size,
                conflict: record.conflict_policy,
                status: status.into(),
            });
            records.push(record);
        }
    }
    let truncated = total_count > records.len();
    if truncated {
        warnings.push(format!(
            "Preview dibatasi ke {limit} dari {total_count} file yang cocok"
        ));
    }
    state.database.insert_preview(&plan_id, &records)?;
    Ok(PreviewResponseDto {
        plan_id,
        items: preview_items,
        total_count,
        truncated,
        warnings,
    })
}

pub fn execute_plan_impl(
    app: &AppHandle,
    state: &AppState,
    request: ExecutePlanRequest,
) -> AppResult<BatchSummaryDto> {
    let items = state
        .database
        .begin_plan(&request.plan_id, &request.item_ids)?;
    for item in items {
        let operation_id = item.id.clone();
        let _ = state.files.execute_operation(item);
        if let Ok(persisted) = state.database.get_operation(&operation_id) {
            let _ = app.emit("amf://activity", persisted.activity());
        }
    }
    let summary = state.database.finish_batch(&request.plan_id)?;
    let _ = app.emit("amf://batch-progress", &summary);
    Ok(summary)
}

pub fn run_manual_from_tray(app: AppHandle, state: AppState) {
    tauri::async_runtime::spawn_blocking(move || {
        let preview = match preview_impl(&state, PreviewRulesRequest::default()) {
            Ok(value) => value,
            Err(_) => return,
        };
        let _ = execute_plan_impl(
            &app,
            &state,
            ExecutePlanRequest {
                plan_id: preview.plan_id,
                item_ids: Vec::new(),
            },
        );
    });
}

fn get_bootstrap_from_state(state: &AppState) -> AppResult<BootstrapDto> {
    let activity = state.database.list_activity(ActivityQuery {
        limit: Some(20),
        ..ActivityQuery::default()
    })?;
    Ok(BootstrapDto {
        rules: state.database.list_rules()?,
        settings: state.database.settings()?,
        monitoring: state.watcher.health(),
        recent_activity: activity.items,
    })
}

fn refresh_monitoring(app: &AppHandle, state: &AppState) -> AppResult<()> {
    if state.database.settings()?.monitoring_enabled {
        state
            .watcher
            .restart(app.clone(), state.database.clone(), state.files.clone())?;
    }
    Ok(())
}

fn apply_autostart(app: &AppHandle, enabled: bool) -> AppResult<()> {
    let manager = app.autolaunch();
    let result = if enabled {
        manager.enable()
    } else {
        manager.disable()
    };
    result.map_err(|error| {
        AppError::new(
            "AUTOSTART_UPDATE_FAILED",
            format!("Could not update automatic startup: {error}"),
        )
    })
}

fn validated_config_path(value: &str) -> AppResult<PathBuf> {
    let path = PathBuf::from(value.trim());
    if !path.is_absolute() {
        return Err(AppError::new(
            "CONFIG_PATH_NOT_ABSOLUTE",
            "Configuration path must be absolute",
        ));
    }
    if path.extension().and_then(|value| value.to_str()) != Some("json") {
        return Err(AppError::new(
            "CONFIG_PATH_EXTENSION",
            "Configuration file must use the .json extension",
        ));
    }
    Ok(path)
}

fn atomic_replace(path: &Path, bytes: &[u8]) -> AppResult<()> {
    let parent = path.parent().ok_or_else(|| {
        AppError::new(
            "CONFIG_PATH_INVALID",
            "Configuration path needs a parent folder",
        )
    })?;
    fs::create_dir_all(parent)?;
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("config.json");
    let nonce = uuid::Uuid::new_v4();
    let temporary = parent.join(format!(".{file_name}.{nonce}.tmp"));
    let backup = parent.join(format!(".{file_name}.{nonce}.bak"));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)?;
    file.write_all(bytes)?;
    file.flush()?;
    file.sync_all()?;
    drop(file);

    let had_existing = path.exists();
    if had_existing {
        fs::rename(path, &backup)?;
    }
    if let Err(error) = fs::rename(&temporary, path) {
        if had_existing {
            let _ = fs::rename(&backup, path);
        }
        let _ = fs::remove_file(&temporary);
        return Err(AppError::new(
            "CONFIG_COMMIT_ERROR",
            format!("Could not save configuration: {error}"),
        ));
    }
    if had_existing {
        let _ = fs::remove_file(backup);
    }
    if let Ok(parent_file) = File::open(parent) {
        let _ = parent_file.sync_all();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_path_must_be_absolute_json() {
        assert_eq!(
            validated_config_path("relative.json")
                .expect_err("relative path")
                .code,
            "CONFIG_PATH_NOT_ABSOLUTE"
        );
        let absolute = if cfg!(windows) {
            "C:\\temp\\config.txt"
        } else {
            "/tmp/config.txt"
        };
        assert_eq!(
            validated_config_path(absolute)
                .expect_err("wrong extension")
                .code,
            "CONFIG_PATH_EXTENSION"
        );
    }
}
