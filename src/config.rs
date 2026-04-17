use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub search_roots: Vec<String>,
    pub max_entries: usize,
    pub exclude_list: HashSet<String>,
    pub project_markers: Vec<String>,
    pub preview_on_jump: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            search_roots: vec![
                "~".to_string(),
                "~/Downloads".to_string(),
                "~/Documents".to_string(),
                "~/Desktop".to_string(),
            ],
            max_entries: 1000,
            exclude_list: vec![
                "node_modules".to_string(),
                ".git".to_string(),
                "__pycache__".to_string(),
                ".venv".to_string(),
                ".next".to_string(),
                ".pytest_cache".to_string(),
                ".casbin".to_string(),
            ]
            .into_iter()
            .collect(),
            project_markers: vec![
                ".git".to_string(),
                "package.json".to_string(),
                "requirements.txt".to_string(),
                "Cargo.toml".to_string(),
                "go.mod".to_string(),
                "pom.xml".to_string(),
                "build.gradle".to_string(),
            ],
            preview_on_jump: true,
        }
    }
}

pub fn get_config() -> Config {
    let mut config_dir = match dirs::config_dir() {
        Some(dir) => dir,
        None => return Config::default(), // Fallback
    };
    config_dir.push("proton-t");

    let config_file = config_dir.join("config.json");

    if !config_dir.exists() {
        let _ = fs::create_dir_all(&config_dir);
    }

    if !config_file.exists() {
        let default_cfg = Config::default();
        if let Ok(json) = serde_json::to_string_pretty(&default_cfg) {
            let _ = fs::write(&config_file, json);
        }
        return default_cfg;
    }

    match fs::read_to_string(&config_file) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(c) => c,
            Err(_) => Config::default(),
        },
        Err(_) => Config::default(),
    }
}
