use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ConflictPolicy {
    #[default]
    AutoRename,
    Skip,
    SafeOverwrite,
}

impl ConflictPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AutoRename => "autoRename",
            Self::Skip => "skip",
            Self::SafeOverwrite => "safeOverwrite",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "skip" => Self::Skip,
            "safeOverwrite" => Self::SafeOverwrite,
            _ => Self::AutoRename,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleSourceDto {
    pub path: String,
    pub canonical_path: String,
    pub available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleDto {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub priority: i64,
    pub sources: Vec<RuleSourceDto>,
    pub destination: String,
    pub extensions: Vec<String>,
    pub name_globs: Vec<String>,
    pub min_size_bytes: Option<u64>,
    pub max_size_bytes: Option<u64>,
    pub conflict_policy: ConflictPolicy,
    pub recursive: bool,
    pub notifications: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertRuleRequest {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub priority: Option<i64>,
    pub sources: Vec<String>,
    pub destination: String,
    #[serde(default)]
    pub extensions: Vec<String>,
    #[serde(default)]
    pub name_globs: Vec<String>,
    #[serde(default)]
    pub min_size_bytes: Option<u64>,
    #[serde(default)]
    pub max_size_bytes: Option<u64>,
    #[serde(default)]
    pub conflict_policy: ConflictPolicy,
    #[serde(default)]
    pub recursive: bool,
    #[serde(default = "default_true")]
    pub notifications: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsDto {
    pub theme: String,
    pub language: String,
    pub minimize_to_tray_on_close: bool,
    pub auto_start: bool,
    pub notifications_enabled: bool,
    pub monitoring_enabled: bool,
    pub onboarding_complete: bool,
    pub history_retention_days: u32,
    pub stability_window_ms: u64,
    pub preview_before_manual: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPatch {
    pub theme: Option<String>,
    pub language: Option<String>,
    pub minimize_to_tray_on_close: Option<bool>,
    pub auto_start: Option<bool>,
    pub notifications_enabled: Option<bool>,
    pub monitoring_enabled: Option<bool>,
    pub onboarding_complete: Option<bool>,
    pub history_retention_days: Option<u32>,
    pub stability_window_ms: Option<u64>,
    pub preview_before_manual: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitoringHealthDto {
    pub state: String,
    pub watched_source_count: usize,
    pub queue_depth: usize,
    pub active_jobs: usize,
    pub last_error: Option<String>,
}

impl Default for MonitoringHealthDto {
    fn default() -> Self {
        Self {
            state: "stopped".into(),
            watched_source_count: 0,
            queue_depth: 0,
            active_jobs: 0,
            last_error: None,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewRulesRequest {
    #[serde(default)]
    pub rule_ids: Vec<String>,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewItemDto {
    pub id: String,
    pub rule_id: String,
    pub rule_name: String,
    pub source_path: String,
    pub destination_path: String,
    pub size_bytes: u64,
    pub conflict: ConflictPolicy,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewResponseDto {
    pub plan_id: String,
    pub items: Vec<PreviewItemDto>,
    pub total_count: usize,
    pub truncated: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutePlanRequest {
    pub plan_id: String,
    #[serde(default)]
    pub item_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchSummaryDto {
    pub batch_id: String,
    pub status: String,
    pub total_count: usize,
    pub completed_count: usize,
    pub failed_count: usize,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityQuery {
    pub cursor: Option<String>,
    pub limit: Option<usize>,
    pub rule_id: Option<String>,
    pub status: Option<String>,
    pub query: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityDto {
    pub id: String,
    pub batch_id: String,
    pub rule_id: Option<String>,
    pub rule_name: Option<String>,
    pub trigger: String,
    pub source_path: String,
    pub destination_path: String,
    pub recovery_path: Option<String>,
    pub status: String,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub detected_at: String,
    pub completed_at: Option<String>,
    pub undone_at: Option<String>,
    pub can_undo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityPageDto {
    pub items: Vec<ActivityDto>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteUndoRequest {
    pub operation_id: String,
    #[serde(default)]
    pub conflict_policy: ConflictPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapDto {
    pub rules: Vec<RuleDto>,
    pub settings: SettingsDto,
    pub monitoring: MonitoringHealthDto,
    pub recent_activity: Vec<ActivityDto>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteRuleRequest {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetRuleEnabledRequest {
    pub id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderRulesRequest {
    pub ordered_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetMonitoringRequest {
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettingsRequest {
    pub settings: SettingsPatch,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigPathRequest {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigExport {
    pub schema_version: u32,
    pub exported_at: String,
    pub rules: Vec<RuleDto>,
    pub settings: SettingsDto,
}

#[derive(Debug, Clone)]
pub struct OperationRecord {
    pub id: String,
    pub batch_id: String,
    pub rule_id: Option<String>,
    pub rule_name: Option<String>,
    pub trigger: String,
    pub source_path: String,
    pub destination_path: String,
    pub recovery_path: Option<String>,
    pub staging_path: Option<String>,
    pub conflict_policy: ConflictPolicy,
    pub source_size: u64,
    pub source_mtime: i64,
    pub source_hash: Option<String>,
    pub destination_hash: Option<String>,
    pub idempotency_key: String,
    pub status: String,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub detected_at: String,
    pub completed_at: Option<String>,
    pub undone_at: Option<String>,
}

impl OperationRecord {
    pub fn activity(&self) -> ActivityDto {
        ActivityDto {
            id: self.id.clone(),
            batch_id: self.batch_id.clone(),
            rule_id: self.rule_id.clone(),
            rule_name: self.rule_name.clone(),
            trigger: self.trigger.clone(),
            source_path: self.source_path.clone(),
            destination_path: self.destination_path.clone(),
            recovery_path: self.recovery_path.clone(),
            status: self.status.clone(),
            error_code: self.error_code.clone(),
            error_message: self.error_message.clone(),
            detected_at: self.detected_at.clone(),
            completed_at: self.completed_at.clone(),
            undone_at: self.undone_at.clone(),
            can_undo: self.status == "completed"
                && self.undone_at.is_none()
                && self.trigger != "undo",
        }
    }
}

pub fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}
