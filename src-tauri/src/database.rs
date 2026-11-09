use std::collections::HashSet;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use rusqlite::{params, Connection, OptionalExtension, Row, Transaction};

use crate::error::{AppError, AppResult};
use crate::models::{
    now_rfc3339, ActivityPageDto, ActivityQuery, BatchSummaryDto, ConflictPolicy, OperationRecord,
    RuleDto, RuleSourceDto, SettingsDto, SettingsPatch,
};

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(path: &Path) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.busy_timeout(Duration::from_secs(5))?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON;",
        )?;
        conn.execute_batch(include_str!("../migrations/0001_init.sql"))?;
        apply_settings_v2_migration(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn lock(&self) -> AppResult<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| AppError::new("DATABASE_LOCK_POISONED", "Database lock is unavailable"))
    }

    pub fn list_rules(&self) -> AppResult<Vec<RuleDto>> {
        let conn = self.lock()?;
        list_rules_conn(&conn)
    }

    pub fn get_rule(&self, id: &str) -> AppResult<Option<RuleDto>> {
        Ok(self.list_rules()?.into_iter().find(|rule| rule.id == id))
    }

    pub fn next_priority(&self) -> AppResult<i64> {
        let conn = self.lock()?;
        Ok(conn.query_row(
            "SELECT COALESCE(MAX(priority), -1) + 1 FROM rules",
            [],
            |row| row.get(0),
        )?)
    }

    pub fn save_rule(&self, rule: &RuleDto) -> AppResult<()> {
        let mut conn = self.lock()?;
        let tx = conn.transaction()?;
        write_rule(&tx, rule)?;
        tx.commit()?;
        Ok(())
    }

    pub fn replace_config(&self, rules: &[RuleDto], settings: &SettingsDto) -> AppResult<()> {
        validate_settings(settings)?;
        let mut conn = self.lock()?;
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM rules", [])?;
        for rule in rules {
            write_rule(&tx, rule)?;
        }
        write_settings(&tx, settings)?;
        tx.commit()?;
        Ok(())
    }

    pub fn delete_rule(&self, id: &str) -> AppResult<()> {
        let conn = self.lock()?;
        if conn.execute("DELETE FROM rules WHERE id = ?1", [id])? == 0 {
            return Err(AppError::new("RULE_NOT_FOUND", "Rule was not found"));
        }
        Ok(())
    }

    pub fn set_rule_enabled(&self, id: &str, enabled: bool) -> AppResult<()> {
        let conn = self.lock()?;
        let now = now_rfc3339();
        if conn.execute(
            "UPDATE rules SET enabled = ?1, updated_at = ?2 WHERE id = ?3",
            params![enabled as i64, now, id],
        )? == 0
        {
            return Err(AppError::new("RULE_NOT_FOUND", "Rule was not found"));
        }
        Ok(())
    }

    pub fn reorder_rules(&self, ids: &[String]) -> AppResult<()> {
        let mut conn = self.lock()?;
        let existing: HashSet<String> = {
            let mut stmt = conn.prepare("SELECT id FROM rules")?;
            let values = stmt
                .query_map([], |row| row.get(0))?
                .collect::<Result<_, _>>()?;
            values
        };
        let supplied: HashSet<&str> = ids.iter().map(String::as_str).collect();
        if ids.len() != supplied.len()
            || existing.len() != supplied.len()
            || !existing.iter().all(|id| supplied.contains(id.as_str()))
        {
            return Err(AppError::new(
                "INVALID_RULE_ORDER",
                "orderedIds must contain every rule exactly once",
            ));
        }
        let tx = conn.transaction()?;
        let now = now_rfc3339();
        for (priority, id) in ids.iter().enumerate() {
            tx.execute(
                "UPDATE rules SET priority = ?1, updated_at = ?2 WHERE id = ?3",
                params![priority as i64, now, id],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn settings(&self) -> AppResult<SettingsDto> {
        let conn = self.lock()?;
        read_settings(&conn)
    }

    pub fn update_settings(&self, patch: SettingsPatch) -> AppResult<SettingsDto> {
        let mut value = self.settings()?;
        if let Some(v) = patch.theme {
            value.theme = v;
        }
        if let Some(v) = patch.language {
            value.language = v;
        }
        if let Some(v) = patch.minimize_to_tray_on_close {
            value.minimize_to_tray_on_close = v;
        }
        if let Some(v) = patch.auto_start {
            value.auto_start = v;
        }
        if let Some(v) = patch.notifications_enabled {
            value.notifications_enabled = v;
        }
        if let Some(v) = patch.monitoring_enabled {
            value.monitoring_enabled = v;
        }
        if let Some(v) = patch.onboarding_complete {
            value.onboarding_complete = v;
        }
        if let Some(v) = patch.history_retention_days {
            value.history_retention_days = v;
        }
        if let Some(v) = patch.stability_window_ms {
            value.stability_window_ms = v;
        }
        if let Some(v) = patch.preview_before_manual {
            value.preview_before_manual = v;
        }
        validate_settings(&value)?;
        let mut conn = self.lock()?;
        let tx = conn.transaction()?;
        write_settings(&tx, &value)?;
        tx.commit()?;
        Ok(value)
    }

    pub fn prune_activity(&self, retention_days: u32) -> AppResult<usize> {
        let conn = self.lock()?;
        let cutoff = chrono::Utc::now() - chrono::Duration::days(i64::from(retention_days));
        Ok(conn.execute(
            "DELETE FROM operation_batches WHERE completed_at IS NOT NULL AND completed_at < ?1",
            [cutoff.to_rfc3339()],
        )?)
    }

    pub fn insert_preview(&self, batch_id: &str, items: &[OperationRecord]) -> AppResult<()> {
        let mut conn = self.lock()?;
        let tx = conn.transaction()?;
        tx.execute(
            "INSERT INTO operation_batches(id,trigger,status,started_at,total_count) VALUES(?1,'manual','preview',?2,?3)",
            params![batch_id, now_rfc3339(), items.len() as i64],
        )?;
        for item in items {
            insert_operation(&tx, item)?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn insert_realtime_item(&self, item: &OperationRecord) -> AppResult<bool> {
        let mut conn = self.lock()?;
        let tx = conn.transaction()?;
        tx.execute(
            "INSERT INTO operation_batches(id,trigger,status,started_at,total_count) VALUES(?1,?2,'running',?3,1)",
            params![item.batch_id, item.trigger, now_rfc3339()],
        )?;
        match insert_operation(&tx, item) {
            Ok(()) => {
                tx.commit()?;
                Ok(true)
            }
            Err(error) if error.code == "DUPLICATE_OPERATION" => Ok(false),
            Err(error) => Err(error),
        }
    }

    pub fn insert_undo_item(&self, item: &OperationRecord) -> AppResult<()> {
        let mut conn = self.lock()?;
        let tx = conn.transaction()?;
        tx.execute(
            "INSERT INTO operation_batches(id,trigger,status,started_at,total_count) VALUES(?1,'undo','running',?2,1)",
            params![item.batch_id, now_rfc3339()],
        )?;
        insert_operation(&tx, item)?;
        tx.commit()?;
        Ok(())
    }

    pub fn begin_plan(
        &self,
        batch_id: &str,
        selected: &[String],
    ) -> AppResult<Vec<OperationRecord>> {
        let mut conn = self.lock()?;
        let tx = conn.transaction()?;
        let batch_status: Option<String> = tx
            .query_row(
                "SELECT status FROM operation_batches WHERE id=?1",
                [batch_id],
                |row| row.get(0),
            )
            .optional()?;
        if batch_status.as_deref() != Some("preview") {
            return Err(AppError::new(
                "PLAN_NOT_FOUND",
                "Preview plan is unavailable or has already been executed",
            ));
        }
        let mut items = {
            let mut stmt = tx.prepare(&format!(
                "{} WHERE batch_id = ?1 AND status = 'planned' ORDER BY detected_at, id",
                operation_select()
            ))?;
            let values = stmt
                .query_map([batch_id], operation_from_row)?
                .collect::<Result<Vec<_>, _>>()?;
            values
        };
        if !selected.is_empty() {
            let selected_ids: HashSet<&str> = selected.iter().map(String::as_str).collect();
            if selected_ids.len() != selected.len()
                || !selected_ids
                    .iter()
                    .all(|id| items.iter().any(|item| item.id == *id))
            {
                return Err(AppError::new(
                    "INVALID_PLAN_SELECTION",
                    "One or more selected plan items are unavailable",
                ));
            }
            for item in items
                .iter()
                .filter(|item| !selected_ids.contains(item.id.as_str()))
            {
                tx.execute("DELETE FROM operation_items WHERE id=?1", [&item.id])?;
            }
            items.retain(|item| selected_ids.contains(item.id.as_str()));
        }
        tx.execute(
            "UPDATE operation_batches SET status='running',total_count=?1 WHERE id=?2",
            params![items.len() as i64, batch_id],
        )?;
        tx.commit()?;
        Ok(items)
    }

    pub fn get_operation(&self, id: &str) -> AppResult<OperationRecord> {
        let conn = self.lock()?;
        conn.query_row(
            &format!("{} WHERE id = ?1", operation_select()),
            [id],
            operation_from_row,
        )
        .optional()?
        .ok_or_else(|| AppError::new("OPERATION_NOT_FOUND", "Operation was not found"))
    }

    pub fn save_operation_if_status(
        &self,
        item: &OperationRecord,
        expected: &[&str],
    ) -> AppResult<()> {
        let conn = self.lock()?;
        let current: Option<String> = conn
            .query_row(
                "SELECT status FROM operation_items WHERE id=?1",
                [&item.id],
                |row| row.get(0),
            )
            .optional()?;
        let current = current
            .ok_or_else(|| AppError::new("OPERATION_NOT_FOUND", "Operation was not found"))?;
        if !expected.contains(&current.as_str()) {
            return Err(AppError::new(
                "INVALID_OPERATION_STATE",
                format!(
                    "Operation {} is in state {current}; expected {}",
                    item.id,
                    expected.join(" or ")
                ),
            ));
        }
        let changed = conn.execute(
            "UPDATE operation_items SET source_path=?1,destination_path=?2,recovery_path=?3,staging_path=?4,source_hash=?5,destination_hash=?6,status=?7,error_code=?8,error_message=?9,completed_at=?10,undone_at=?11,updated_at=?12 WHERE id=?13 AND status=?14",
            params![
                item.source_path,
                item.destination_path,
                item.recovery_path,
                item.staging_path,
                item.source_hash,
                item.destination_hash,
                item.status,
                item.error_code,
                item.error_message,
                item.completed_at,
                item.undone_at,
                now_rfc3339(),
                item.id,
                current,
            ],
        )?;
        if changed != 1 {
            return Err(
                AppError::new("OPERATION_RACE", "Operation state changed concurrently").retryable(),
            );
        }
        Ok(())
    }

    pub fn finish_batch(&self, id: &str) -> AppResult<BatchSummaryDto> {
        let conn = self.lock()?;
        let (total, complete, failed): (i64, i64, i64) = conn.query_row(
            "SELECT COUNT(*),
                    COALESCE(SUM(CASE WHEN status IN ('completed','skipped','undone') THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN status IN ('failed','needsReview') THEN 1 ELSE 0 END), 0)
             FROM operation_items WHERE batch_id=?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        let status = if total == 0 {
            "completed"
        } else if failed == 0 {
            "completed"
        } else if complete == 0 {
            "failed"
        } else {
            "partial"
        };
        conn.execute(
            "UPDATE operation_batches SET status=?1,completed_at=?2,total_count=?3,completed_count=?4,failed_count=?5 WHERE id=?6",
            params![status, now_rfc3339(), total, complete, failed, id],
        )?;
        Ok(BatchSummaryDto {
            batch_id: id.to_string(),
            status: status.into(),
            total_count: total as usize,
            completed_count: complete as usize,
            failed_count: failed as usize,
        })
    }

    pub fn list_activity(&self, query: ActivityQuery) -> AppResult<ActivityPageDto> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(&format!(
            "{} WHERE status != 'planned' ORDER BY detected_at DESC, id DESC LIMIT 1000",
            operation_select()
        ))?;
        let mut records = stmt
            .query_map([], operation_from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        if let Some(cursor) = query.cursor {
            records.retain(|item| item.detected_at < cursor);
        }
        if let Some(rule_id) = query.rule_id {
            records.retain(|item| item.rule_id.as_deref() == Some(rule_id.as_str()));
        }
        if let Some(status) = query.status {
            records.retain(|item| item.status == status);
        }
        if let Some(text) = query.query {
            let text = text.to_lowercase();
            records.retain(|item| {
                item.source_path.to_lowercase().contains(&text)
                    || item.destination_path.to_lowercase().contains(&text)
                    || item
                        .rule_name
                        .as_deref()
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(&text)
            });
        }
        let limit = query.limit.unwrap_or(50).clamp(1, 200);
        let has_more = records.len() > limit;
        records.truncate(limit);
        let next_cursor = if has_more {
            records.last().map(|item| item.detected_at.clone())
        } else {
            None
        };
        Ok(ActivityPageDto {
            items: records.into_iter().map(|item| item.activity()).collect(),
            next_cursor,
        })
    }

    pub fn mark_incomplete_for_review(&self) -> AppResult<usize> {
        let conn = self.lock()?;
        Ok(conn.execute(
            "UPDATE operation_items
             SET status='needsReview',
                 error_code='INTERRUPTED_OPERATION',
                 error_message='The app stopped before this operation completed',
                 updated_at=?1
             WHERE status IN ('pending','staging','staged','destinationCommitted','undoPending')",
            [now_rfc3339()],
        )?)
    }

    pub fn complete_undo(&self, undo: &OperationRecord, original_id: &str) -> AppResult<()> {
        let mut conn = self.lock()?;
        let tx = conn.transaction()?;
        let changed_child = tx.execute(
            "UPDATE operation_items SET source_path=?1,destination_path=?2,
                    recovery_path=?3,staging_path=?4,source_hash=?5,destination_hash=?6,
                    status='completed',error_code=NULL,error_message=NULL,completed_at=?7,
                    updated_at=?8
             WHERE id=?9 AND status='staging'",
            params![
                undo.source_path,
                undo.destination_path,
                undo.recovery_path,
                undo.staging_path,
                undo.source_hash,
                undo.destination_hash,
                undo.completed_at,
                now_rfc3339(),
                undo.id,
            ],
        )?;
        let now = now_rfc3339();
        let changed_original = tx.execute(
            "UPDATE operation_items SET status='undone',undone_at=?1,updated_at=?1
             WHERE id=?2 AND status='completed' AND undone_at IS NULL",
            params![now, original_id],
        )?;
        if changed_child != 1 || changed_original != 1 {
            return Err(AppError::new(
                "UNDO_COMMIT_RACE",
                "Undo state changed concurrently",
            ));
        }
        tx.commit()?;
        Ok(())
    }
}

fn apply_settings_v2_migration(conn: &Connection) -> AppResult<()> {
    let applied: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version=2)",
        [],
        |row| row.get(0),
    )?;
    if !applied {
        conn.execute_batch(include_str!("../migrations/0002_settings_v2.sql"))?;
    }
    Ok(())
}

fn validate_settings(value: &SettingsDto) -> AppResult<()> {
    if !matches!(value.theme.as_str(), "system" | "light" | "dark") {
        return Err(AppError::new(
            "INVALID_THEME",
            "Theme must be system, light, or dark",
        ));
    }
    if !matches!(value.language.as_str(), "id" | "en") {
        return Err(AppError::new(
            "INVALID_LANGUAGE",
            "Language must be id or en",
        ));
    }
    if !(1..=3650).contains(&value.history_retention_days) {
        return Err(AppError::new(
            "INVALID_RETENTION",
            "History retention must be between 1 and 3650 days",
        ));
    }
    if !(250..=30_000).contains(&value.stability_window_ms) {
        return Err(AppError::new(
            "INVALID_STABILITY_WINDOW",
            "Stability window must be between 250 and 30000 milliseconds",
        ));
    }
    Ok(())
}

fn read_settings(conn: &Connection) -> AppResult<SettingsDto> {
    Ok(conn.query_row(
        "SELECT theme,language,minimize_to_tray_on_close,auto_start,
                notifications_enabled,monitoring_enabled,onboarding_complete,
                history_retention_days,stability_window_ms,preview_before_manual
         FROM settings WHERE id=1",
        [],
        |row| {
            Ok(SettingsDto {
                theme: row.get(0)?,
                language: row.get(1)?,
                minimize_to_tray_on_close: row.get::<_, i64>(2)? != 0,
                auto_start: row.get::<_, i64>(3)? != 0,
                notifications_enabled: row.get::<_, i64>(4)? != 0,
                monitoring_enabled: row.get::<_, i64>(5)? != 0,
                onboarding_complete: row.get::<_, i64>(6)? != 0,
                history_retention_days: row.get::<_, i64>(7)? as u32,
                stability_window_ms: row.get::<_, i64>(8)? as u64,
                preview_before_manual: row.get::<_, i64>(9)? != 0,
            })
        },
    )?)
}

fn write_settings(tx: &Transaction<'_>, value: &SettingsDto) -> AppResult<()> {
    tx.execute(
        "UPDATE settings SET theme=?1,language=?2,minimize_to_tray_on_close=?3,
                auto_start=?4,notifications_enabled=?5,monitoring_enabled=?6,
                onboarding_complete=?7,history_retention_days=?8,
                stability_window_ms=?9,preview_before_manual=?10,updated_at=?11
         WHERE id=1",
        params![
            value.theme,
            value.language,
            value.minimize_to_tray_on_close as i64,
            value.auto_start as i64,
            value.notifications_enabled as i64,
            value.monitoring_enabled as i64,
            value.onboarding_complete as i64,
            i64::from(value.history_retention_days),
            value.stability_window_ms as i64,
            value.preview_before_manual as i64,
            now_rfc3339(),
        ],
    )?;
    Ok(())
}

fn write_rule(tx: &Transaction<'_>, rule: &RuleDto) -> AppResult<()> {
    tx.execute(
        "INSERT INTO rules(
            id,name,enabled,priority,destination,conflict_policy,recursive,
            notifications,min_size_bytes,max_size_bytes,created_at,updated_at
         ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)
         ON CONFLICT(id) DO UPDATE SET
            name=excluded.name,enabled=excluded.enabled,priority=excluded.priority,
            destination=excluded.destination,conflict_policy=excluded.conflict_policy,
            recursive=excluded.recursive,notifications=excluded.notifications,
            min_size_bytes=excluded.min_size_bytes,max_size_bytes=excluded.max_size_bytes,
            updated_at=excluded.updated_at",
        params![
            rule.id,
            rule.name,
            rule.enabled as i64,
            rule.priority,
            rule.destination,
            rule.conflict_policy.as_str(),
            rule.recursive as i64,
            rule.notifications as i64,
            rule.min_size_bytes.map(|v| v as i64),
            rule.max_size_bytes.map(|v| v as i64),
            rule.created_at,
            rule.updated_at,
        ],
    )?;
    tx.execute("DELETE FROM rule_sources WHERE rule_id=?1", [&rule.id])?;
    tx.execute("DELETE FROM rule_extensions WHERE rule_id=?1", [&rule.id])?;
    tx.execute("DELETE FROM rule_globs WHERE rule_id=?1", [&rule.id])?;
    for source in &rule.sources {
        tx.execute(
            "INSERT INTO rule_sources(id,rule_id,path,canonical_path,available)
             VALUES(?1,?2,?3,?4,?5)",
            params![
                uuid::Uuid::new_v4().to_string(),
                rule.id,
                source.path,
                source.canonical_path,
                source.available as i64,
            ],
        )?;
    }
    for extension in &rule.extensions {
        tx.execute(
            "INSERT INTO rule_extensions(rule_id,value) VALUES(?1,?2)",
            params![rule.id, extension],
        )?;
    }
    for glob in &rule.name_globs {
        tx.execute(
            "INSERT INTO rule_globs(rule_id,value) VALUES(?1,?2)",
            params![rule.id, glob],
        )?;
    }
    Ok(())
}

fn list_rules_conn(conn: &Connection) -> AppResult<Vec<RuleDto>> {
    type RuleBase = (
        String,
        String,
        bool,
        i64,
        String,
        String,
        bool,
        bool,
        Option<i64>,
        Option<i64>,
        String,
        String,
    );
    let mut stmt = conn.prepare(
        "SELECT id,name,enabled,priority,destination,conflict_policy,recursive,
                notifications,min_size_bytes,max_size_bytes,created_at,updated_at
         FROM rules ORDER BY priority,created_at,id",
    )?;
    let base: Vec<RuleBase> = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get::<_, i64>(2)? != 0,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get::<_, i64>(6)? != 0,
                row.get::<_, i64>(7)? != 0,
                row.get(8)?,
                row.get(9)?,
                row.get(10)?,
                row.get(11)?,
            ))
        })?
        .collect::<Result<_, _>>()?;
    let mut result = Vec::with_capacity(base.len());
    for (
        id,
        name,
        enabled,
        priority,
        destination,
        policy,
        recursive,
        notifications,
        min,
        max,
        created_at,
        updated_at,
    ) in base
    {
        let sources = {
            let mut stmt = conn.prepare(
                "SELECT path,canonical_path,available FROM rule_sources
                 WHERE rule_id=?1 ORDER BY rowid",
            )?;
            let values = stmt
                .query_map([&id], |row| {
                    Ok(RuleSourceDto {
                        path: row.get(0)?,
                        canonical_path: row.get(1)?,
                        available: row.get::<_, i64>(2)? != 0,
                    })
                })?
                .collect::<Result<_, _>>()?;
            values
        };
        result.push(RuleDto {
            id: id.clone(),
            name,
            enabled,
            priority,
            sources,
            destination,
            extensions: child_values(conn, "rule_extensions", &id)?,
            name_globs: child_values(conn, "rule_globs", &id)?,
            min_size_bytes: min.map(|v| v as u64),
            max_size_bytes: max.map(|v| v as u64),
            conflict_policy: ConflictPolicy::parse(&policy),
            recursive,
            notifications,
            created_at,
            updated_at,
        });
    }
    Ok(result)
}

fn child_values(conn: &Connection, table: &str, rule_id: &str) -> AppResult<Vec<String>> {
    let sql = match table {
        "rule_extensions" => "SELECT value FROM rule_extensions WHERE rule_id=?1 ORDER BY value",
        "rule_globs" => "SELECT value FROM rule_globs WHERE rule_id=?1 ORDER BY value",
        _ => {
            return Err(AppError::new(
                "DATABASE_QUERY_ERROR",
                "Unsupported child table",
            ))
        }
    };
    let mut stmt = conn.prepare(sql)?;
    let values = stmt
        .query_map([rule_id], |row| row.get(0))?
        .collect::<Result<_, _>>()?;
    Ok(values)
}

fn insert_operation(tx: &Transaction<'_>, item: &OperationRecord) -> AppResult<()> {
    tx.execute(
        "INSERT INTO operation_items(
            id,batch_id,rule_id,rule_name,trigger,source_path,destination_path,
            recovery_path,staging_path,conflict_policy,source_size,source_mtime,
            source_hash,destination_hash,idempotency_key,status,error_code,
            error_message,detected_at,completed_at,undone_at,updated_at
         ) VALUES(
            ?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,
            ?17,?18,?19,?20,?21,?22
         )",
        params![
            item.id,
            item.batch_id,
            item.rule_id,
            item.rule_name,
            item.trigger,
            item.source_path,
            item.destination_path,
            item.recovery_path,
            item.staging_path,
            item.conflict_policy.as_str(),
            item.source_size as i64,
            item.source_mtime,
            item.source_hash,
            item.destination_hash,
            item.idempotency_key,
            item.status,
            item.error_code,
            item.error_message,
            item.detected_at,
            item.completed_at,
            item.undone_at,
            now_rfc3339(),
        ],
    )
    .map(|_| ())
    .map_err(|error| {
        if error.to_string().contains("idx_operation_idempotency")
            || error.to_string().contains("UNIQUE constraint failed")
        {
            AppError::new(
                "DUPLICATE_OPERATION",
                "This file snapshot is already being processed",
            )
        } else {
            error.into()
        }
    })
}

fn operation_select() -> &'static str {
    "SELECT id,batch_id,rule_id,rule_name,trigger,source_path,destination_path,
            recovery_path,staging_path,conflict_policy,source_size,source_mtime,
            source_hash,destination_hash,idempotency_key,status,error_code,
            error_message,detected_at,completed_at,undone_at
     FROM operation_items"
}

fn operation_from_row(row: &Row<'_>) -> rusqlite::Result<OperationRecord> {
    Ok(OperationRecord {
        id: row.get(0)?,
        batch_id: row.get(1)?,
        rule_id: row.get(2)?,
        rule_name: row.get(3)?,
        trigger: row.get(4)?,
        source_path: row.get(5)?,
        destination_path: row.get(6)?,
        recovery_path: row.get(7)?,
        staging_path: row.get(8)?,
        conflict_policy: ConflictPolicy::parse(&row.get::<_, String>(9)?),
        source_size: row.get::<_, i64>(10)? as u64,
        source_mtime: row.get(11)?,
        source_hash: row.get(12)?,
        destination_hash: row.get(13)?,
        idempotency_key: row.get(14)?,
        status: row.get(15)?,
        error_code: row.get(16)?,
        error_message: row.get(17)?,
        detected_at: row.get(18)?,
        completed_at: row.get(19)?,
        undone_at: row.get(20)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_creates_v2_settings() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = Database::open(&dir.path().join("app.db")).expect("database");
        let settings = db.settings().expect("settings");
        assert_eq!(settings.theme, "system");
        assert_eq!(settings.history_retention_days, 30);
        assert_eq!(settings.stability_window_ms, 1500);
        assert!(settings.preview_before_manual);
        assert!(!settings.monitoring_enabled);
    }

    #[test]
    fn settings_validation_is_enforced() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = Database::open(&dir.path().join("app.db")).expect("database");
        let result = db.update_settings(SettingsPatch {
            stability_window_ms: Some(1),
            ..SettingsPatch::default()
        });
        assert_eq!(
            result.expect_err("invalid setting").code,
            "INVALID_STABILITY_WINDOW"
        );
    }
}
