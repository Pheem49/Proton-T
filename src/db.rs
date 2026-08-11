use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::path::Path;
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
            .unwrap_or_default()
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

    write_json_atomic(&db_path, &db);
}

/// Acquires an exclusive, cross-process file lock, then loads, mutates, and
/// (if `mutate` reports a change) saves the database as one atomic critical
/// section. This prevents two concurrent `proton-t` invocations (e.g. two
/// shells `cd`-ing at once) from racing on a read-modify-write and silently
/// dropping each other's update.
pub fn with_locked_db<F>(max_entries: usize, mutate: F)
where
    F: FnOnce(&mut HashMap<String, Entry>) -> bool,
{
    let _lock = acquire_lock();
    let mut db = load_db();
    if mutate(&mut db) {
        save_db(db, max_entries);
    }
}

fn acquire_lock() -> Option<File> {
    locked_file(".proton_t_db.lock")
}

/// Opens (creating if needed) the named file under the home directory and
/// takes an exclusive, blocking OS-level lock on it. The lock is released
/// when the returned `File` is dropped. Shared by `db` and `bookmarks` so
/// both stores get the same cross-process read-modify-write protection.
pub(crate) fn locked_file(name: &str) -> Option<File> {
    let mut lock_path = dirs::home_dir()?;
    lock_path.push(name);
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(lock_path)
        .ok()?;
    file.lock().ok()?;
    Some(file)
}

/// Serializes `value` to pretty JSON and writes it to `path` via a
/// write-to-temp-then-rename so a crash mid-write can never leave a
/// half-written file in place (rename is atomic on Linux).
pub(crate) fn write_json_atomic<T: Serialize>(path: &Path, value: &T) {
    if let Ok(json) = serde_json::to_string_pretty(value) {
        let tmp_path = path.with_extension("json.tmp");
        if fs::write(&tmp_path, json).is_ok() {
            let _ = fs::rename(&tmp_path, path);
        }
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
