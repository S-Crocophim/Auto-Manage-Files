use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub watch_folder: String,
    pub extensions: String,
    pub destination: String,
    pub enabled: bool,
    #[serde(default = "default_conflict_action")]
    pub conflict_action: String, // "rename", "skip", "overwrite"
    pub pattern: Option<String>,
    pub min_size_mb: Option<f64>,
    pub max_size_mb: Option<f64>,
}

fn default_conflict_action() -> String {
    "rename".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoveHistory {
    pub id: String,
    pub rule_name: String,
    pub src_path: String,
    pub dest_path: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub auto_start: bool,
    pub minimize_to_tray_on_close: bool,
    pub theme: String,
    pub language: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_start: false,
            minimize_to_tray_on_close: true,
            theme: "dark".to_string(),
            language: "id".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub rules: Vec<Rule>,
    pub settings: Settings,
    #[serde(default)]
    pub history: Vec<MoveHistory>,
}

pub fn get_config_path() -> PathBuf {
    let mut dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("AutoFileOrganizer");
    fs::create_dir_all(&dir).ok();
    dir.push("config.json");
    dir
}

pub fn default_config() -> AppConfig {
    let downloads = dirs::download_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let documents = dirs::document_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let pictures = dirs::picture_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    AppConfig {
        rules: vec![
            Rule {
                id: Uuid::new_v4().to_string()[..8].to_string(),
                name: "Spreadsheets".to_string(),
                watch_folder: downloads.clone(),
                extensions: ".xlsx, .xls, .csv".to_string(),
                destination: format!("{}/Excel Files", documents),
                enabled: true,
                conflict_action: "rename".to_string(),
                pattern: None,
                min_size_mb: None,
                max_size_mb: None,
            },
            Rule {
                id: Uuid::new_v4().to_string()[..8].to_string(),
                name: "Images".to_string(),
                watch_folder: downloads.clone(),
                extensions: ".jpg, .png, .gif".to_string(),
                destination: format!("{}/Downloaded Images", pictures),
                enabled: true,
                conflict_action: "rename".to_string(),
                pattern: None,
                min_size_mb: None,
                max_size_mb: None,
            },
            Rule {
                id: Uuid::new_v4().to_string()[..8].to_string(),
                name: "Documents".to_string(),
                watch_folder: downloads,
                extensions: ".pdf, .docx, .txt".to_string(),
                destination: format!("{}/Downloaded Docs", documents),
                enabled: true,
                conflict_action: "rename".to_string(),
                pattern: None,
                min_size_mb: None,
                max_size_mb: None,
            },
        ],
        settings: Settings::default(),
        history: Vec::new(),
    }
}

pub fn load_config() -> AppConfig {
    let path = get_config_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                return config;
            }
        }
    }
    let config = default_config();
    save_config(&config).ok();
    config
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = get_config_path();
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}
