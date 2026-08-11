use crate::db::{locked_file, write_json_atomic};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn bookmarks_path() -> Option<PathBuf> {
    let mut path = dirs::home_dir()?;
    path.push(".proton_t_bookmarks.json");
    Some(path)
}

pub fn load_bookmarks() -> HashMap<String, String> {
    let Some(path) = bookmarks_path() else {
        return HashMap::new();
    };

    if !path.exists() {
        return HashMap::new();
    }

    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}

fn save_bookmarks(bookmarks: &HashMap<String, String>) {
    if let Some(path) = bookmarks_path() {
        write_json_atomic(&path, bookmarks);
    }
}

/// Same locked read-modify-write pattern as `db::with_locked_db`, scoped to
/// the bookmarks file so concurrent `save`/`unsave` calls can't race.
pub fn with_locked_bookmarks<F>(mutate: F)
where
    F: FnOnce(&mut HashMap<String, String>) -> bool,
{
    let _lock = locked_file(".proton_t_bookmarks.lock");
    let mut bookmarks = load_bookmarks();
    if mutate(&mut bookmarks) {
        save_bookmarks(&bookmarks);
    }
}
