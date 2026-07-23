use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use crate::config::Rule;

pub fn safe_dest_path(dest_folder: &Path, filename: &str) -> PathBuf {
    let path = Path::new(filename);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(filename);
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let ext_str = if ext.is_empty() {
        String::new()
    } else {
        format!(".{}", ext)
    };

    let mut candidate = dest_folder.join(filename);
    let mut counter = 1;

    while candidate.exists() {
        let new_name = format!("{} ({}){}", stem, counter, ext_str);
        candidate = dest_folder.join(new_name);
        counter += 1;
    }

    candidate
}

pub fn matches_rule_criteria(path: &Path, rule: &Rule) -> bool {
    if !path.is_file() {
        return false;
    }

    // Check extension
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let formatted_ext = format!(".{}", ext.to_lowercase());
        let rule_exts: Vec<String> = rule
            .extensions
            .split(',')
            .map(|e| e.trim().to_lowercase())
            .filter(|e| !e.is_empty())
            .collect();

        if !rule_exts.contains(&formatted_ext) && !rule_exts.contains(&ext.to_lowercase()) {
            return false;
        }
    } else {
        return false;
    }

    // Check filename pattern (optional)
    if let Some(ref pat) = rule.pattern {
        let pat_trim = pat.trim().to_lowercase();
        if !pat_trim.is_empty() {
            let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("").to_lowercase();
            // Wildcard or simple substring match
            let clean_pat = pat_trim.replace('*', "");
            if !filename.contains(&clean_pat) {
                return false;
            }
        }
    }

    // Check file size in MB (optional)
    if let Ok(metadata) = fs::metadata(path) {
        let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
        if let Some(min_s) = rule.min_size_mb {
            if size_mb < min_s {
                return false;
            }
        }
        if let Some(max_s) = rule.max_size_mb {
            if size_mb > max_s {
                return false;
            }
        }
    }

    true
}

pub fn move_file_with_conflict(
    src_path: &Path,
    dest_folder: &Path,
    conflict_action: &str,
    max_retries: u32,
    retry_delay_ms: u64,
) -> Result<PathBuf, String> {
    if !src_path.is_file() {
        return Err(format!("Source is not a valid file: {:?}", src_path));
    }

    fs::create_dir_all(dest_folder).map_err(|e| format!("Failed to create dest dir: {}", e))?;

    let filename = src_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Invalid filename".to_string())?;

    let dest_path = match conflict_action {
        "skip" => {
            let candidate = dest_folder.join(filename);
            if candidate.exists() {
                return Err(format!("Skipped: file already exists at {:?}", candidate));
            }
            candidate
        }
        "overwrite" => {
            let candidate = dest_folder.join(filename);
            if candidate.exists() {
                fs::remove_file(&candidate).ok();
            }
            candidate
        }
        _ => safe_dest_path(dest_folder, filename), // default "rename"
    };

    for attempt in 1..=max_retries {
        match fs::rename(src_path, &dest_path) {
            Ok(_) => return Ok(dest_path),
            Err(_) => {
                if fs::copy(src_path, &dest_path).is_ok() {
                    if fs::remove_file(src_path).is_ok() {
                        return Ok(dest_path);
                    }
                }
                if attempt < max_retries {
                    thread::sleep(Duration::from_millis(retry_delay_ms));
                }
            }
        }
    }

    Err(format!(
        "Failed to move file after {} retries: {:?}",
        max_retries, src_path
    ))
}

pub fn undo_file_move(src_path_str: &str, dest_path_str: &str) -> Result<(), String> {
    let current_path = PathBuf::from(dest_path_str);
    let target_path = PathBuf::from(src_path_str);

    if !current_path.is_file() {
        return Err(format!("File to undo does not exist at: {}", dest_path_str));
    }

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    match fs::rename(&current_path, &target_path) {
        Ok(_) => Ok(()),
        Err(_) => {
            fs::copy(&current_path, &target_path).map_err(|e| e.to_string())?;
            fs::remove_file(&current_path).map_err(|e| e.to_string())?;
            Ok(())
        }
    }
}

pub fn run_manual_organization(rules: &[Rule]) -> usize {
    let mut total_moved = 0;

    for rule in rules {
        if !rule.enabled {
            continue;
        }

        let dest_folder = PathBuf::from(rule.destination.trim());
        let watch_folders: Vec<&str> = rule.watch_folder.split(',').map(|w| w.trim()).collect();

        for wf in watch_folders {
            let watch_path = PathBuf::from(wf);
            if !watch_path.is_dir() {
                continue;
            }

            if let Ok(entries) = fs::read_dir(watch_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if matches_rule_criteria(&path, rule) {
                        if move_file_with_conflict(&path, &dest_folder, &rule.conflict_action, 3, 500).is_ok() {
                            total_moved += 1;
                        }
                    }
                }
            }
        }
    }

    total_moved
}
