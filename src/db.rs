use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Entry {
    pub score: f64,
    pub last_access: f64,
    #[serde(default)]
    pub is_project: bool,
    #[serde(default)]
    pub removed: bool,
}

pub fn load_db() -> HashMap<String, Entry> {
    let mut db_path = match dirs::home_dir() {
        Some(dir) => dir,
        None => return HashMap::new(),
    };
    db_path.push(".proton_t_db.json");

    if !db_path.exists() {
        return HashMap::new();
    }

    match fs::read_to_string(&db_path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}

pub fn save_db(mut db: HashMap<String, Entry>, max_entries: usize) {
    let mut db_path = match dirs::home_dir() {
        Some(dir) => dir,
        None => return,
    };
    db_path.push(".proton_t_db.json");

    if db.len() > max_entries {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        let mut removed_entries = Vec::new();
        let mut active_entries = Vec::new();
        for item in db.into_iter() {
            if item.1.removed {
                removed_entries.push(item);
            } else {
                active_entries.push(item);
            }
        }
        active_entries.sort_by(|a, b| {
            let score_b = get_score(&b.1, now);
            let score_a = get_score(&a.1, now);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let active_limit = max_entries.saturating_sub(removed_entries.len());
        active_entries.truncate(active_limit);
        removed_entries.extend(active_entries);
        db = removed_entries.into_iter().collect();
    }

    if let Ok(json) = serde_json::to_string_pretty(&db) {
        let _ = fs::write(db_path, json);
    }
}

pub fn get_score(entry: &Entry, now: f64) -> f64 {
    let mut time_diff = now - entry.last_access;
    if time_diff < 0.0 {
        time_diff = 0.0;
    }
    // 604800 seconds = 1 week half-life
    let decay = (-time_diff / 604800.0).exp();
    let mut final_score = entry.score * decay;
    if entry.is_project {
        final_score *= 1.2;
    }
    final_score
}
