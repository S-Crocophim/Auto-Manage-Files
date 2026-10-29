use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::UNIX_EPOCH;

use crate::database::Database;
use crate::error::{AppError, AppResult};
use crate::models::{now_rfc3339, ActivityDto, ConflictPolicy, OperationRecord};
use crate::rules::available_path;

pub struct FileService {
    database: Arc<Database>,
    recovery_root: PathBuf,
    mutation_lock: Mutex<()>,
}

impl FileService {
    pub fn new(database: Arc<Database>, recovery_root: PathBuf) -> AppResult<Self> {
        fs::create_dir_all(&recovery_root)?;
        Ok(Self {
            database,
            recovery_root,
            mutation_lock: Mutex::new(()),
        })
    }

    fn lock_mutations(&self) -> AppResult<MutexGuard<'_, ()>> {
        self.mutation_lock.lock().map_err(|_| {
            AppError::new(
                "FILE_OPERATION_LOCK_POISONED",
                "File operations are temporarily unavailable",
            )
        })
    }

    pub fn execute_operation(&self, mut item: OperationRecord) -> AppResult<OperationRecord> {
        let _guard = self.lock_mutations()?;
        self.execute_locked(&mut item)?;
        Ok(item)
    }

    pub fn execute_undo(
        &self,
        original_id: &str,
        conflict_policy: ConflictPolicy,
    ) -> AppResult<ActivityDto> {
        let _guard = self.lock_mutations()?;
        let original = self.database.get_operation(original_id)?;
        if original.status != "completed" || original.undone_at.is_some() {
            return Err(AppError::new(
                "UNDO_NOT_AVAILABLE",
                "This operation cannot be undone",
            ));
        }

        let destination = PathBuf::from(&original.destination_path);
        let requested_source = PathBuf::from(&original.source_path);
        if !destination.is_file() {
            return Err(AppError::new(
                "UNDO_SOURCE_MISSING",
                "The organized file no longer exists at its destination",
            ));
        }
        if let Some(expected_hash) = original.destination_hash.as_deref() {
            let actual_hash = hash_file(&destination)?;
            if actual_hash != expected_hash {
                return Err(AppError::new(
                    "UNDO_FILE_CHANGED",
                    "The destination file changed after it was organized",
                ));
            }
        }

        let batch_id = uuid::Uuid::new_v4().to_string();
        let mut target = requested_source.clone();
        let mut displaced_source = None;
        if target.exists() {
            match conflict_policy {
                ConflictPolicy::AutoRename => target = available_path(&target),
                ConflictPolicy::Skip => {
                    let mut undo =
                        make_undo_record(&original, &batch_id, target, conflict_policy, "pending")?;
                    self.database.insert_undo_item(&undo)?;
                    undo.status = "skipped".into();
                    undo.completed_at = Some(now_rfc3339());
                    undo.error_code = Some("UNDO_TARGET_EXISTS".into());
                    undo.error_message =
                        Some("Undo skipped because the original path is occupied".into());
                    self.database
                        .save_operation_if_status(&undo, &["pending"])?;
                    self.database.finish_batch(&batch_id)?;
                    return Ok(undo.activity());
                }
                ConflictPolicy::SafeOverwrite => {
                    displaced_source =
                        Some(self.recovery_path(&format!("undo-{batch_id}"), target.file_name()));
                }
            }
        }

        let mut undo = make_undo_record(
            &original,
            &batch_id,
            target.clone(),
            conflict_policy,
            "pending",
        )?;
        undo.recovery_path = displaced_source
            .as_ref()
            .map(|path| path.to_string_lossy().to_string());
        undo.staging_path = Some(
            staging_path(&target, &undo.id)
                .to_string_lossy()
                .to_string(),
        );
        self.database.insert_undo_item(&undo)?;
        undo.status = "staging".into();
        self.database
            .save_operation_if_status(&undo, &["pending"])?;

        let result = (|| -> AppResult<()> {
            if let Some(recovery) = displaced_source.as_deref() {
                transfer_verified(
                    &target,
                    recovery,
                    &staging_path(recovery, &format!("source-{}", undo.id)),
                )?;
            }
            transfer_verified(&destination, &target, &staging_path(&target, &undo.id))?;

            if let Some(original_recovery) = original.recovery_path.as_deref() {
                let recovery = Path::new(original_recovery);
                if recovery.exists() {
                    transfer_verified(
                        recovery,
                        &destination,
                        &staging_path(&destination, &format!("restore-{}", undo.id)),
                    )?;
                }
            }
            Ok(())
        })();

        match result {
            Ok(()) => {
                undo.status = "completed".into();
                undo.source_hash = original.destination_hash.clone();
                undo.destination_hash = Some(hash_file(&target)?);
                undo.completed_at = Some(now_rfc3339());
                self.database.complete_undo(&undo, &original.id)?;
                self.database.finish_batch(&batch_id)?;
                Ok(undo.activity())
            }
            Err(error) => {
                rollback_undo(
                    &destination,
                    &target,
                    displaced_source.as_deref(),
                    original.recovery_path.as_deref().map(Path::new),
                    &undo.id,
                );
                undo.status = if destination.exists() && target.exists() {
                    "needsReview".into()
                } else {
                    "failed".into()
                };
                undo.error_code = Some(error.code.clone());
                undo.error_message = Some(error.message.clone());
                undo.completed_at = Some(now_rfc3339());
                let _ = self.database.save_operation_if_status(&undo, &["staging"]);
                let _ = self.database.finish_batch(&batch_id);
                Err(error.with_operation(undo.id))
            }
        }
    }

    fn execute_locked(&self, item: &mut OperationRecord) -> AppResult<()> {
        let source = PathBuf::from(&item.source_path);
        let metadata = source.metadata().map_err(|error| {
            AppError::new(
                "SOURCE_UNAVAILABLE",
                format!("Cannot access source {}: {error}", source.display()),
            )
        })?;
        if !metadata.is_file() {
            return self.fail_before_mutation(
                item,
                "SOURCE_NOT_FILE",
                "Source is not a regular file",
            );
        }
        let current_mtime = metadata_mtime(&metadata);
        if metadata.len() != item.source_size || current_mtime != item.source_mtime {
            return self.fail_before_mutation(
                item,
                "SOURCE_CHANGED",
                "Source size or modification time changed after it was planned",
            );
        }

        let mut destination = PathBuf::from(&item.destination_path);
        match item.conflict_policy {
            ConflictPolicy::AutoRename => destination = available_path(&destination),
            ConflictPolicy::Skip if destination.exists() => {
                item.status = "skipped".into();
                item.completed_at = Some(now_rfc3339());
                item.error_code = Some("DESTINATION_EXISTS".into());
                item.error_message =
                    Some("File skipped because the destination already exists".into());
                self.database
                    .save_operation_if_status(item, &["planned", "pending"])?;
                return Ok(());
            }
            ConflictPolicy::Skip | ConflictPolicy::SafeOverwrite => {}
        }
        let parent = destination.parent().ok_or_else(|| {
            AppError::new(
                "INVALID_DESTINATION",
                "Destination must have a parent folder",
            )
        })?;
        fs::create_dir_all(parent)?;

        let mut staging = staging_path(&destination, &item.id);
        let recovery =
            if item.conflict_policy == ConflictPolicy::SafeOverwrite && destination.exists() {
                Some(self.recovery_path(&item.id, destination.file_name()))
            } else {
                None
            };

        item.destination_path = destination.to_string_lossy().to_string();
        item.staging_path = Some(staging.to_string_lossy().to_string());
        item.recovery_path = recovery
            .as_ref()
            .map(|path| path.to_string_lossy().to_string());
        item.status = "staging".into();
        self.database
            .save_operation_if_status(item, &["planned", "pending"])?;

        let source_hash = hash_file(&source)?;
        item.source_hash = Some(source_hash.clone());
        self.database.save_operation_if_status(item, &["staging"])?;

        let mut result = (|| -> AppResult<()> {
            if let Some(recovery) = recovery.as_deref() {
                transfer_verified(
                    &destination,
                    recovery,
                    &staging_path(recovery, &format!("backup-{}", item.id)),
                )?;
            }
            transfer_verified(&source, &destination, &staging)?;
            Ok(())
        })();
        if item.conflict_policy == ConflictPolicy::AutoRename {
            for _ in 0..8 {
                let should_retry = result
                    .as_ref()
                    .is_err_and(|error| error.code == "TRANSFER_DESTINATION_EXISTS")
                    && source.exists();
                if !should_retry {
                    break;
                }
                destination = available_path(&destination);
                staging = staging_path(&destination, &item.id);
                item.destination_path = destination.to_string_lossy().to_string();
                item.staging_path = Some(staging.to_string_lossy().to_string());
                self.database.save_operation_if_status(item, &["staging"])?;
                result = transfer_verified(&source, &destination, &staging);
            }
        }

        match result {
            Ok(()) => {
                item.destination_hash = Some(hash_file(&destination)?);
                item.status = "completed".into();
                item.completed_at = Some(now_rfc3339());
                item.staging_path = None;
                self.database.save_operation_if_status(item, &["staging"])?;
                Ok(())
            }
            Err(error) => {
                if source.exists() && !destination.exists() {
                    if let Some(recovery) = recovery.as_deref() {
                        let _ = transfer_verified(
                            recovery,
                            &destination,
                            &staging_path(&destination, &format!("rollback-{}", item.id)),
                        );
                    }
                }
                item.status = if source.exists() == destination.exists() {
                    "needsReview".into()
                } else {
                    "failed".into()
                };
                item.error_code = Some(error.code.clone());
                item.error_message = Some(error.message.clone());
                item.completed_at = Some(now_rfc3339());
                let _ = self.database.save_operation_if_status(item, &["staging"]);
                Err(error.with_operation(item.id.clone()))
            }
        }
    }

    fn fail_before_mutation<T>(
        &self,
        item: &mut OperationRecord,
        code: &str,
        message: &str,
    ) -> AppResult<T> {
        item.status = "failed".into();
        item.error_code = Some(code.into());
        item.error_message = Some(message.into());
        item.completed_at = Some(now_rfc3339());
        self.database
            .save_operation_if_status(item, &["planned", "pending"])?;
        Err(AppError::new(code, message).with_operation(item.id.clone()))
    }

    fn recovery_path(&self, operation_id: &str, file_name: Option<&std::ffi::OsStr>) -> PathBuf {
        let file_name = file_name
            .and_then(|value| value.to_str())
            .unwrap_or("recovered-file");
        let safe_name = file_name.replace(['/', '\\', ':'], "_");
        self.recovery_root
            .join(format!("{operation_id}-{safe_name}.bak"))
    }
}

fn make_undo_record(
    original: &OperationRecord,
    batch_id: &str,
    destination: PathBuf,
    conflict_policy: ConflictPolicy,
    status: &str,
) -> AppResult<OperationRecord> {
    let source = Path::new(&original.destination_path);
    let metadata = source.metadata()?;
    Ok(OperationRecord {
        id: uuid::Uuid::new_v4().to_string(),
        batch_id: batch_id.to_string(),
        rule_id: original.rule_id.clone(),
        rule_name: original.rule_name.clone(),
        trigger: "undo".into(),
        source_path: original.destination_path.clone(),
        destination_path: destination.to_string_lossy().to_string(),
        recovery_path: None,
        staging_path: None,
        conflict_policy,
        source_size: metadata.len(),
        source_mtime: original.source_mtime,
        source_hash: original.destination_hash.clone(),
        destination_hash: None,
        idempotency_key: format!("undo:{}", original.id),
        status: status.into(),
        error_code: None,
        error_message: None,
        detected_at: now_rfc3339(),
        completed_at: None,
        undone_at: None,
    })
}

fn transfer_verified(source: &Path, destination: &Path, staging: &Path) -> AppResult<()> {
    if !source.is_file() {
        return Err(AppError::new(
            "TRANSFER_SOURCE_MISSING",
            format!("Transfer source does not exist: {}", source.display()),
        ));
    }
    if destination.exists() {
        return Err(AppError::new(
            "TRANSFER_DESTINATION_EXISTS",
            format!(
                "Transfer destination already exists: {}",
                destination.display()
            ),
        ));
    }
    let parent = destination.parent().ok_or_else(|| {
        AppError::new(
            "INVALID_DESTINATION",
            "Transfer destination must have a parent folder",
        )
    })?;
    fs::create_dir_all(parent)?;

    match fs::hard_link(source, destination) {
        Ok(()) => {
            if let Err(error) = fs::remove_file(source) {
                return Err(AppError::new(
                    "SOURCE_DELETE_FAILED",
                    format!("Destination was linked, but source deletion failed: {error}"),
                ));
            }
            sync_parent(destination);
            return Ok(());
        }
        Err(link_error) => {
            if staging.exists() {
                return Err(AppError::new(
                    "STAGING_PATH_OCCUPIED",
                    format!("Staging path is occupied: {}", staging.display()),
                ));
            }
            let copy_result = copy_and_verify(source, staging);
            if let Err(error) = copy_result {
                let _ = fs::remove_file(staging);
                return Err(error);
            }
            if let Err(error) = commit_staging(staging, destination) {
                let _ = fs::remove_file(staging);
                if destination.exists() {
                    return Err(AppError::new(
                        "TRANSFER_DESTINATION_EXISTS",
                        format!(
                            "Transfer destination appeared while committing: {}",
                            destination.display()
                        ),
                    )
                    .retryable());
                }
                return Err(AppError::new(
                    "DESTINATION_COMMIT_FAILED",
                    format!(
                        "Could not commit copied file after direct link failed ({link_error}): {error}"
                    ),
                )
                .retryable());
            }
            sync_parent(destination);
            if let Err(error) = fs::remove_file(source) {
                return Err(AppError::new(
                    "SOURCE_DELETE_FAILED",
                    format!(
                        "Verified destination was created, but source deletion failed: {error}"
                    ),
                ));
            }
            sync_parent(source);
        }
    }
    Ok(())
}

fn commit_staging(staging: &Path, destination: &Path) -> io::Result<()> {
    match fs::hard_link(staging, destination) {
        Ok(()) => fs::remove_file(staging),
        Err(_link_error) => {
            #[cfg(windows)]
            {
                fs::rename(staging, destination)
            }
            #[cfg(not(windows))]
            {
                Err(_link_error)
            }
        }
    }
}

fn copy_and_verify(source: &Path, staging: &Path) -> AppResult<()> {
    let mut input = File::open(source)?;
    let mut output = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(staging)?;
    io::copy(&mut input, &mut output)?;
    output.flush()?;
    output.sync_all()?;
    let source_size = source.metadata()?.len();
    let staging_size = staging.metadata()?.len();
    if source_size != staging_size {
        return Err(AppError::new(
            "COPY_SIZE_MISMATCH",
            "Copied file size does not match its source",
        ));
    }
    let source_hash = hash_file(source)?;
    let staging_hash = hash_file(staging)?;
    if source_hash != staging_hash {
        return Err(AppError::new(
            "COPY_HASH_MISMATCH",
            "Copied file checksum does not match its source",
        ));
    }
    Ok(())
}

fn hash_file(path: &Path) -> AppResult<String> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

fn metadata_mtime(metadata: &fs::Metadata) -> i64 {
    metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|value| value.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or_default()
}

fn staging_path(destination: &Path, operation_id: &str) -> PathBuf {
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    parent.join(format!(".amf-{operation_id}.part"))
}

fn sync_parent(path: &Path) {
    if let Some(parent) = path.parent() {
        if let Ok(directory) = File::open(parent) {
            let _ = directory.sync_all();
        }
    }
}

fn rollback_undo(
    original_destination: &Path,
    restored_source: &Path,
    displaced_source: Option<&Path>,
    original_recovery: Option<&Path>,
    operation_id: &str,
) {
    if restored_source.exists() && !original_destination.exists() {
        let _ = transfer_verified(
            restored_source,
            original_destination,
            &staging_path(
                original_destination,
                &format!("undo-rollback-{operation_id}"),
            ),
        );
    }
    if let Some(recovery) = displaced_source {
        if recovery.exists() && !restored_source.exists() {
            let _ = transfer_verified(
                recovery,
                restored_source,
                &staging_path(restored_source, &format!("source-rollback-{operation_id}")),
            );
        }
    }
    if let Some(recovery) = original_recovery {
        if recovery.exists() && !original_destination.exists() {
            let _ = transfer_verified(
                recovery,
                original_destination,
                &staging_path(
                    original_destination,
                    &format!("dest-rollback-{operation_id}"),
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ConflictPolicy;

    fn operation(source: &Path, destination: &Path, policy: ConflictPolicy) -> OperationRecord {
        let metadata = source.metadata().expect("metadata");
        OperationRecord {
            id: uuid::Uuid::new_v4().to_string(),
            batch_id: uuid::Uuid::new_v4().to_string(),
            rule_id: None,
            rule_name: Some("test".into()),
            trigger: "manual".into(),
            source_path: source.display().to_string(),
            destination_path: destination.display().to_string(),
            recovery_path: None,
            staging_path: None,
            conflict_policy: policy,
            source_size: metadata.len(),
            source_mtime: metadata_mtime(&metadata),
            source_hash: None,
            destination_hash: None,
            idempotency_key: uuid::Uuid::new_v4().to_string(),
            status: "pending".into(),
            error_code: None,
            error_message: None,
            detected_at: now_rfc3339(),
            completed_at: None,
            undone_at: None,
        }
    }

    #[test]
    fn verified_transfer_moves_content() {
        let dir = tempfile::tempdir().expect("tempdir");
        let source = dir.path().join("source.txt");
        let destination = dir.path().join("nested").join("destination.txt");
        std::fs::write(&source, b"verified").expect("source");
        transfer_verified(&source, &destination, &dir.path().join("stage")).expect("transfer");
        assert!(!source.exists());
        assert_eq!(
            std::fs::read(&destination).expect("destination"),
            b"verified"
        );
    }

    #[test]
    fn safe_overwrite_keeps_recovery_and_supports_undo() {
        let dir = tempfile::tempdir().expect("tempdir");
        let database = Arc::new(Database::open(&dir.path().join("app.db")).expect("database"));
        let service =
            FileService::new(database.clone(), dir.path().join("recovery")).expect("service");
        let source = dir.path().join("source.txt");
        let destination = dir.path().join("destination.txt");
        std::fs::write(&source, b"new").expect("source");
        std::fs::write(&destination, b"old").expect("destination");
        let item = operation(&source, &destination, ConflictPolicy::SafeOverwrite);
        database
            .insert_realtime_item(&item)
            .expect("insert operation");
        let completed = service.execute_operation(item).expect("execute");
        assert_eq!(
            std::fs::read(&destination).expect("new destination"),
            b"new"
        );
        assert!(completed
            .recovery_path
            .as_deref()
            .is_some_and(|path| Path::new(path).exists()));
        let undo = service
            .execute_undo(&completed.id, ConflictPolicy::AutoRename)
            .expect("undo");
        assert_eq!(undo.status, "completed");
        assert_eq!(std::fs::read(&source).expect("restored source"), b"new");
        assert_eq!(
            std::fs::read(&destination).expect("restored destination"),
            b"old"
        );
    }
}
