use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_search_roots")]
    pub search_roots: Vec<String>,
    #[serde(default = "default_max_entries")]
    pub max_entries: usize,
    #[serde(default = "default_exclude_list")]
    pub exclude_list: HashSet<String>,
    #[serde(default = "default_project_markers")]
    pub project_markers: Vec<String>,
    #[serde(default = "default_preview_on_jump")]
    pub preview_on_jump: bool,
}

fn default_search_roots() -> Vec<String> {
    vec![
        "~".to_string(),
        "~/Downloads".to_string(),
        "~/Documents".to_string(),
        "~/Desktop".to_string(),
    ]
}

fn default_max_entries() -> usize {
    1000
}

fn default_exclude_list() -> HashSet<String> {
    vec![
        "node_modules".to_string(),
        ".git".to_string(),
        "__pycache__".to_string(),
        ".venv".to_string(),
        ".next".to_string(),
        ".pytest_cache".to_string(),
        ".casbin".to_string(),
    ]
    .into_iter()
    .collect()
}

fn default_project_markers() -> Vec<String> {
    vec![
        ".git".to_string(),
        "package.json".to_string(),
        "requirements.txt".to_string(),
        "Cargo.toml".to_string(),
        "go.mod".to_string(),
        "pom.xml".to_string(),
        "build.gradle".to_string(),
    ]
}

fn default_preview_on_jump() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Config {
            search_roots: default_search_roots(),
            max_entries: default_max_entries(),
            exclude_list: default_exclude_list(),
            project_markers: default_project_markers(),
            preview_on_jump: default_preview_on_jump(),
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
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn deserializing_older_config_keeps_defaults_for_new_fields() {
        let parsed: Config = serde_json::from_str(
            r#"{
                "search_roots": ["~/Code"],
                "max_entries": 42,
                "exclude_list": [".git"]
            }"#,
        )
        .expect("config should deserialize");

        assert_eq!(parsed.search_roots, vec!["~/Code"]);
        assert_eq!(parsed.max_entries, 42);
        assert!(parsed.exclude_list.contains(".git"));
        assert!(parsed.preview_on_jump);
        assert!(parsed.project_markers.contains(&"Cargo.toml".to_string()));
    }
}
