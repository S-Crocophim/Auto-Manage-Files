use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};
use std::time::UNIX_EPOCH;

use globset::{GlobBuilder, GlobSetBuilder};
use walkdir::WalkDir;

use crate::error::{AppError, AppResult};
use crate::models::{
    now_rfc3339, ConflictPolicy, OperationRecord, RuleDto, RuleSourceDto, UpsertRuleRequest,
};

const MAX_RULE_NAME: usize = 120;
const MAX_FILTERS: usize = 64;

pub fn normalize_rule(
    request: UpsertRuleRequest,
    existing: Option<&RuleDto>,
    next_priority: i64,
) -> AppResult<RuleDto> {
    let name = request.name.trim().to_string();
    if name.is_empty() || name.chars().count() > MAX_RULE_NAME {
        return Err(AppError::new(
            "INVALID_RULE_NAME",
            "Rule name must contain between 1 and 120 characters",
        ));
    }
    if request.sources.is_empty() {
        return Err(AppError::new(
            "MISSING_RULE_SOURCE",
            "A rule needs at least one source folder",
        ));
    }
    if request.sources.len() > MAX_FILTERS
        || request.extensions.len() > MAX_FILTERS
        || request.name_globs.len() > MAX_FILTERS
    {
        return Err(AppError::new(
            "TOO_MANY_RULE_FILTERS",
            "A rule supports at most 64 sources, extensions, and name patterns",
        ));
    }
    if let (Some(min), Some(max)) = (request.min_size_bytes, request.max_size_bytes) {
        if min > max {
            return Err(AppError::new(
                "INVALID_SIZE_RANGE",
                "Minimum file size cannot exceed maximum file size",
            ));
        }
    }

    let destination = normalize_destination(Path::new(request.destination.trim()))?;
    reject_critical_path(&destination)?;

    let mut seen_sources = HashSet::new();
    let mut sources = Vec::with_capacity(request.sources.len());
    for raw in request.sources {
        let raw = raw.trim();
        if raw.is_empty() {
            continue;
        }
        let path = PathBuf::from(raw);
        if !path.is_absolute() {
            return Err(AppError::new(
                "SOURCE_NOT_ABSOLUTE",
                format!("Source folder must be an absolute path: {raw}"),
            ));
        }
        if !path.is_dir() {
            return Err(AppError::new(
                "SOURCE_UNAVAILABLE",
                format!("Source folder is unavailable: {raw}"),
            ));
        }
        let canonical = dunce::canonicalize(&path).map_err(|error| {
            AppError::new(
                "SOURCE_UNAVAILABLE",
                format!("Cannot access source folder {raw}: {error}"),
            )
        })?;
        reject_critical_path(&canonical)?;
        let key = comparable_path(&canonical);
        if !seen_sources.insert(key) {
            continue;
        }
        if same_path(&canonical, &destination) || path_is_within(&destination, &canonical) {
            return Err(AppError::new(
                "RULE_PATH_CYCLE",
                "Destination cannot be the source folder or a folder inside it",
            ));
        }
        sources.push(RuleSourceDto {
            path: path.to_string_lossy().to_string(),
            canonical_path: canonical.to_string_lossy().to_string(),
            available: true,
        });
    }
    if sources.is_empty() {
        return Err(AppError::new(
            "MISSING_RULE_SOURCE",
            "A rule needs at least one accessible source folder",
        ));
    }

    let mut extensions = Vec::new();
    for value in request.extensions {
        if let Some(value) = normalize_extension(&value)? {
            extensions.push(value);
        }
    }
    extensions.sort();
    extensions.dedup();

    let mut name_globs = request
        .name_globs
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    name_globs.sort();
    name_globs.dedup();
    compile_globs(&name_globs)?;

    let now = now_rfc3339();
    let id = match request.id.as_deref() {
        Some(value) => uuid::Uuid::parse_str(value)
            .map_err(|_| AppError::new("INVALID_RULE_ID", "Rule id must be a valid UUID"))?
            .to_string(),
        None => uuid::Uuid::new_v4().to_string(),
    };
    Ok(RuleDto {
        id,
        name,
        enabled: request.enabled,
        priority: request
            .priority
            .or_else(|| existing.map(|rule| rule.priority))
            .unwrap_or(next_priority)
            .max(0),
        sources,
        destination: destination.to_string_lossy().to_string(),
        extensions,
        name_globs,
        min_size_bytes: request.min_size_bytes,
        max_size_bytes: request.max_size_bytes,
        conflict_policy: request.conflict_policy,
        recursive: request.recursive,
        notifications: request.notifications,
        created_at: existing
            .map(|rule| rule.created_at.clone())
            .unwrap_or_else(|| now.clone()),
        updated_at: now,
    })
}

pub fn normalize_imported_rule(rule: RuleDto, next_priority: i64) -> AppResult<RuleDto> {
    normalize_rule(
        UpsertRuleRequest {
            id: Some(rule.id),
            name: rule.name,
            enabled: rule.enabled,
            priority: Some(next_priority),
            sources: rule.sources.into_iter().map(|source| source.path).collect(),
            destination: rule.destination,
            extensions: rule.extensions,
            name_globs: rule.name_globs,
            min_size_bytes: rule.min_size_bytes,
            max_size_bytes: rule.max_size_bytes,
            conflict_policy: rule.conflict_policy,
            recursive: rule.recursive,
            notifications: rule.notifications,
        },
        None,
        next_priority,
    )
}

pub fn matching_rule<'a>(path: &Path, rules: &'a [RuleDto]) -> Option<&'a RuleDto> {
    rules
        .iter()
        .filter(|rule| rule.enabled)
        .find(|rule| rule_matches_path(path, rule))
}

pub fn validate_rule_graph(rules: &[RuleDto]) -> AppResult<()> {
    let mut edges = vec![Vec::new(); rules.len()];
    for (from_index, from) in rules.iter().enumerate() {
        let destination = Path::new(&from.destination);
        for (to_index, to) in rules.iter().enumerate() {
            if from_index == to_index || !from.enabled || !to.enabled {
                continue;
            }
            if to
                .sources
                .iter()
                .any(|source| path_is_within(destination, Path::new(&source.canonical_path)))
            {
                edges[from_index].push(to_index);
            }
        }
    }
    let mut colors = vec![0_u8; rules.len()];
    for index in 0..rules.len() {
        if graph_has_cycle(index, &edges, &mut colors) {
            return Err(AppError::new(
                "RULE_GRAPH_CYCLE",
                "Rule destinations form a processing cycle",
            ));
        }
    }
    Ok(())
}

fn graph_has_cycle(index: usize, edges: &[Vec<usize>], colors: &mut [u8]) -> bool {
    if colors[index] == 1 {
        return true;
    }
    if colors[index] == 2 {
        return false;
    }
    colors[index] = 1;
    for &next in &edges[index] {
        if graph_has_cycle(next, edges, colors) {
            return true;
        }
    }
    colors[index] = 2;
    false
}

pub fn rule_matches_path(path: &Path, rule: &RuleDto) -> bool {
    let canonical = match dunce::canonicalize(path) {
        Ok(value) => value,
        Err(_) => return false,
    };
    if !canonical.is_file() {
        return false;
    }
    let Some(file_name) = canonical.file_name().and_then(|value| value.to_str()) else {
        return false;
    };
    let lower_name = file_name.to_lowercase();
    if file_name.starts_with('.')
        || file_name.starts_with("~$")
        || [".tmp", ".part", ".crdownload", ".download"]
            .iter()
            .any(|suffix| lower_name.ends_with(suffix))
    {
        return false;
    }
    let destination = Path::new(&rule.destination);
    if path_is_within(&canonical, destination) {
        return false;
    }
    let source_matches = rule.sources.iter().any(|source| {
        let source_path = Path::new(&source.canonical_path);
        if !path_is_within(&canonical, source_path) {
            return false;
        }
        rule.recursive
            || canonical
                .parent()
                .is_some_and(|parent| same_path(parent, source_path))
    });
    if !source_matches {
        return false;
    }

    if !rule.extensions.is_empty() {
        let extension = canonical
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| format!(".{}", value.to_lowercase()));
        if !extension
            .as_ref()
            .is_some_and(|value| rule.extensions.iter().any(|item| item == value))
        {
            return false;
        }
    }

    if !rule.name_globs.is_empty() {
        let Some(name) = canonical.file_name() else {
            return false;
        };
        let Ok(globs) = compile_globs(&rule.name_globs) else {
            return false;
        };
        if !globs.is_match(name) {
            return false;
        }
    }

    let Ok(metadata) = canonical.metadata() else {
        return false;
    };
    if rule
        .min_size_bytes
        .is_some_and(|minimum| metadata.len() < minimum)
        || rule
            .max_size_bytes
            .is_some_and(|maximum| metadata.len() > maximum)
    {
        return false;
    }
    true
}

pub fn scan_matching_files(
    rule: &RuleDto,
    limit: usize,
    claimed: &mut HashSet<String>,
) -> (Vec<PathBuf>, usize, bool, Vec<String>) {
    let mut files = Vec::new();
    let mut warnings = Vec::new();
    let mut total_count = 0;
    for source in &rule.sources {
        let source_path = Path::new(&source.canonical_path);
        if !source_path.is_dir() {
            warnings.push(format!("Source unavailable: {}", source.path));
            continue;
        }
        let max_depth = if rule.recursive { usize::MAX } else { 1 };
        for entry in WalkDir::new(source_path)
            .follow_links(false)
            .min_depth(1)
            .max_depth(max_depth)
        {
            let entry = match entry {
                Ok(value) => value,
                Err(error) => {
                    warnings.push(error.to_string());
                    continue;
                }
            };
            if !entry.file_type().is_file() || !rule_matches_path(entry.path(), rule) {
                continue;
            }
            let key = comparable_path(entry.path());
            if !claimed.insert(key) {
                continue;
            }
            total_count += 1;
            if files.len() < limit {
                files.push(entry.into_path());
            }
        }
    }
    (files, total_count, total_count > limit, warnings)
}

pub fn build_operation(
    rule: &RuleDto,
    path: &Path,
    batch_id: &str,
    trigger: &str,
    status: &str,
) -> AppResult<OperationRecord> {
    let metadata = path.metadata().map_err(|error| {
        AppError::new(
            "SOURCE_UNAVAILABLE",
            format!("Cannot read source file {}: {error}", path.display()),
        )
    })?;
    if !metadata.is_file() {
        return Err(AppError::new(
            "SOURCE_NOT_FILE",
            "The selected source is not a regular file",
        ));
    }
    let filename = path
        .file_name()
        .ok_or_else(|| AppError::new("INVALID_FILENAME", "Source filename is invalid"))?;
    let raw_destination = Path::new(&rule.destination).join(filename);
    let destination = match rule.conflict_policy {
        ConflictPolicy::AutoRename => available_path(&raw_destination),
        ConflictPolicy::Skip | ConflictPolicy::SafeOverwrite => raw_destination,
    };
    let mtime = metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|value| value.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or_default();
    let idempotency_key = blake3::hash(
        format!(
            "{}\0{}\0{}\0{}",
            comparable_path(path),
            metadata.len(),
            mtime,
            rule.id
        )
        .as_bytes(),
    )
    .to_hex()
    .to_string();
    Ok(OperationRecord {
        id: uuid::Uuid::new_v4().to_string(),
        batch_id: batch_id.to_string(),
        rule_id: Some(rule.id.clone()),
        rule_name: Some(rule.name.clone()),
        trigger: trigger.to_string(),
        source_path: path.to_string_lossy().to_string(),
        destination_path: destination.to_string_lossy().to_string(),
        recovery_path: None,
        staging_path: None,
        conflict_policy: rule.conflict_policy,
        source_size: metadata.len(),
        source_mtime: mtime,
        source_hash: None,
        destination_hash: None,
        idempotency_key,
        status: status.to_string(),
        error_code: None,
        error_message: None,
        detected_at: now_rfc3339(),
        completed_at: None,
        undone_at: None,
    })
}

pub fn available_path(candidate: &Path) -> PathBuf {
    if !candidate.exists() {
        return candidate.to_path_buf();
    }
    let stem = candidate
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("file");
    let suffix = candidate
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| format!(".{value}"))
        .unwrap_or_default();
    let parent = candidate.parent().unwrap_or_else(|| Path::new(""));
    for index in 1..=10_000 {
        let value = parent.join(format!("{stem} ({index}){suffix}"));
        if !value.exists() {
            return value;
        }
    }
    parent.join(format!("{stem} ({}){suffix}", uuid::Uuid::new_v4()))
}

fn normalize_extension(value: &str) -> AppResult<Option<String>> {
    let extension = value.trim().trim_start_matches('.').to_lowercase();
    if extension.is_empty() {
        return Ok(None);
    }
    if extension.len() > 32
        || extension
            .chars()
            .any(|character| matches!(character, '/' | '\\' | '*' | '?' | ':'))
    {
        return Err(AppError::new(
            "INVALID_EXTENSION",
            format!("Invalid file extension filter: {value}"),
        ));
    }
    Ok(Some(format!(".{extension}")))
}

fn compile_globs(values: &[String]) -> AppResult<globset::GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for value in values {
        if value.len() > 256 {
            return Err(AppError::new(
                "INVALID_NAME_PATTERN",
                "Filename pattern cannot exceed 256 characters",
            ));
        }
        let glob = GlobBuilder::new(value)
            .case_insensitive(cfg!(windows))
            .literal_separator(true)
            .build()
            .map_err(|error| {
                AppError::new(
                    "INVALID_NAME_PATTERN",
                    format!("Invalid filename pattern {value}: {error}"),
                )
            })?;
        builder.add(glob);
    }
    builder
        .build()
        .map_err(|error| AppError::new("INVALID_NAME_PATTERN", error.to_string()))
}

fn normalize_destination(path: &Path) -> AppResult<PathBuf> {
    if !path.is_absolute() {
        return Err(AppError::new(
            "DESTINATION_NOT_ABSOLUTE",
            "Destination folder must be an absolute path",
        ));
    }
    let normalized = if path.exists() {
        dunce::canonicalize(path).map_err(|error| {
            AppError::new(
                "DESTINATION_UNAVAILABLE",
                format!("Cannot access destination: {error}"),
            )
        })?
    } else {
        lexical_normalize(path)
    };
    Ok(normalized)
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            value => normalized.push(value.as_os_str()),
        }
    }
    normalized
}

fn reject_critical_path(path: &Path) -> AppResult<()> {
    if path.parent().is_none() {
        return Err(AppError::new(
            "PROTECTED_PATH",
            "Drive and filesystem roots cannot be organized",
        ));
    }
    for variable in [
        "SystemRoot",
        "ProgramFiles",
        "ProgramFiles(x86)",
        "ProgramData",
    ] {
        if let Some(value) = std::env::var_os(variable) {
            let protected = PathBuf::from(value);
            if same_path(path, &protected) || path_is_within(path, &protected) {
                return Err(AppError::new(
                    "PROTECTED_PATH",
                    format!("System folder cannot be organized: {}", path.display()),
                ));
            }
        }
    }
    Ok(())
}

fn path_is_within(path: &Path, parent: &Path) -> bool {
    let path = comparable_path(path);
    let mut parent = comparable_path(parent);
    if path == parent {
        return true;
    }
    if !parent.ends_with(std::path::MAIN_SEPARATOR) {
        parent.push(std::path::MAIN_SEPARATOR);
    }
    path.starts_with(&parent)
}

fn same_path(left: &Path, right: &Path) -> bool {
    comparable_path(left) == comparable_path(right)
}

fn comparable_path(path: &Path) -> String {
    let value = lexical_normalize(path).to_string_lossy().to_string();
    if cfg!(windows) {
        value.to_lowercase()
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(source: &Path, destination: &Path, recursive: bool) -> RuleDto {
        RuleDto {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Images".into(),
            enabled: true,
            priority: 0,
            sources: vec![RuleSourceDto {
                path: source.display().to_string(),
                canonical_path: dunce::canonicalize(source)
                    .expect("canonical source")
                    .display()
                    .to_string(),
                available: true,
            }],
            destination: destination.display().to_string(),
            extensions: vec![".png".into()],
            name_globs: vec!["report-*.png".into()],
            min_size_bytes: None,
            max_size_bytes: None,
            conflict_policy: ConflictPolicy::AutoRename,
            recursive,
            notifications: true,
            created_at: now_rfc3339(),
            updated_at: now_rfc3339(),
        }
    }

    #[test]
    fn matching_is_source_aware_and_honors_recursive_flag() {
        let dir = tempfile::tempdir().expect("tempdir");
        let source = dir.path().join("source");
        let other = dir.path().join("other");
        let nested = source.join("nested");
        let destination = dir.path().join("destination");
        std::fs::create_dir_all(&nested).expect("nested");
        std::fs::create_dir_all(&other).expect("other");
        let direct = source.join("report-a.png");
        let deep = nested.join("report-b.png");
        let outside = other.join("report-c.png");
        std::fs::write(&direct, b"x").expect("direct");
        std::fs::write(&deep, b"x").expect("deep");
        std::fs::write(&outside, b"x").expect("outside");
        let non_recursive = rule(&source, &destination, false);
        assert!(rule_matches_path(&direct, &non_recursive));
        assert!(!rule_matches_path(&deep, &non_recursive));
        assert!(!rule_matches_path(&outside, &non_recursive));
        assert!(rule_matches_path(&deep, &rule(&source, &destination, true)));
    }

    #[test]
    fn available_path_preserves_extension() {
        let dir = tempfile::tempdir().expect("tempdir");
        let candidate = dir.path().join("archive.tar.gz");
        std::fs::write(&candidate, b"x").expect("candidate");
        assert_eq!(
            available_path(&candidate)
                .file_name()
                .and_then(|v| v.to_str()),
            Some("archive.tar (1).gz")
        );
    }
}
