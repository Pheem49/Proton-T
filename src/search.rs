use crate::config::Config;
use crate::db::{get_score, Entry};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Intent {
    pub recent: bool,
    pub project: bool,
    pub kws: Vec<String>,
    pub tags: Vec<String>,
}

fn get_tag_map() -> HashMap<&'static str, Vec<&'static str>> {
    let mut map = HashMap::new();
    map.insert(
        "backend",
        vec!["api", "server", "node", "backend", "go", "java"],
    );
    map.insert(
        "frontend",
        vec!["ui", "react", "web", "frontend", "client", "next", "vue"],
    );
    map
}

pub fn parse_intent(keywords: &[String]) -> Intent {
    let mut intent = Intent {
        recent: false,
        project: false,
        kws: Vec::new(),
        tags: Vec::new(),
    };

    let tag_map = get_tag_map();

    for kw in keywords {
        let kw_lower = kw.to_lowercase();
        if ["last", "recent", "latest", "today"].contains(&kw_lower.as_str()) {
            intent.recent = true;
        } else if ["project", "app", "proj"].contains(&kw_lower.as_str()) {
            intent.project = true;
        } else {
            let mut mapped = false;
            for (tag, syns) in &tag_map {
                if syns.contains(&kw_lower.as_str()) || &kw_lower == tag {
                    for s in syns {
                        intent.tags.push(s.to_string());
                    }
                    mapped = true;
                }
            }
            if !mapped {
                intent.kws.push(kw_lower);
            }
        }
    }
    intent
}

pub fn is_fuzzy_match(keywords: &[String], path: &str) -> bool {
    let path_lower = path.to_lowercase();
    for kw in keywords {
        let kw_lower = kw.to_lowercase();
        if path_lower.contains(&kw_lower) {
            continue;
        }
        let mut it = path_lower.chars();
        let matched = kw_lower
            .chars()
            .all(|k_char| it.by_ref().any(|p_char| p_char == k_char));
        if !matched {
            return false;
        }
    }
    true
}

pub fn match_with_intent(
    path: &str,
    entry: &Entry,
    intent: &Intent,
    project_markers: &[String],
    keywords: &[String],
    now: f64,
) -> (bool, f64) {
    if !intent.kws.is_empty()
        && !is_fuzzy_match(&intent.kws, path)
        && !is_fuzzy_match(keywords, path)
    {
        return (false, 0.0);
    }

    if !intent.tags.is_empty() {
        let path_lower = path.to_lowercase();
        if !intent.tags.iter().any(|t| path_lower.contains(t)) {
            return (false, 0.0);
        }
    }

    if intent.project
        && !entry.is_project
        && !path_has_project_marker(path, project_markers)
        && !path.to_lowercase().contains("project")
    {
        return (false, 0.0);
    }

    let mut score = get_score(entry, now);
    let basename = Path::new(path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();

    if !intent.kws.is_empty()
        && intent
            .kws
            .iter()
            .all(|k| basename.contains(&k.to_lowercase()))
    {
        score *= 10.0;
    }

    if intent.recent {
        let mut time_diff = now - entry.last_access;
        if time_diff < 1.0 {
            time_diff = 1.0;
        }
        score = (1.0 / time_diff) * 1e9;
    }

    (true, score)
}

fn path_has_project_marker(path: &str, project_markers: &[String]) -> bool {
    let Ok(entries) = fs::read_dir(path) else {
        return false;
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if project_markers.contains(&name) {
            return true;
        }
    }

    false
}

fn expand_tilde(p: &str) -> PathBuf {
    if p.starts_with("~/") || p == "~" {
        if let Some(mut home) = dirs::home_dir() {
            if p == "~" {
                return home;
            }
            home.push(&p[2..]);
            return home;
        }
    }
    PathBuf::from(p)
}

pub fn fallback_search(config: &Config, keywords: &[String], limit: usize) -> Vec<String> {
    let mut found_paths = Vec::new();
    let max_depth = 2;

    let roots: Vec<PathBuf> = config
        .search_roots
        .iter()
        .map(|r| expand_tilde(r))
        .filter(|p| p.is_dir())
        .collect();

    for root in roots {
        let mut queue = vec![(root, 0)];
        while !queue.is_empty() {
            let (curr_dir, depth) = queue.remove(0);
            if depth > max_depth {
                continue;
            }
            if let Ok(entries) = fs::read_dir(&curr_dir) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_dir() {
                            let name = entry.file_name().to_string_lossy().into_owned();
                            if name.starts_with('.') || config.exclude_list.contains(&name) {
                                continue;
                            }
                            if is_fuzzy_match(keywords, &name) {
                                let path_str = entry.path().to_string_lossy().into_owned();
                                if !found_paths.contains(&path_str) {
                                    found_paths.push(path_str);
                                    if found_paths.len() >= limit {
                                        return found_paths;
                                    }
                                }
                            }
                            queue.push((entry.path(), depth + 1));
                        }
                    }
                }
            }
        }
    }
    found_paths
}

pub fn fallback_project_search(config: &Config, intent: &Intent, limit: usize) -> Vec<String> {
    let mut found_paths = Vec::new();
    let max_depth = 2;

    let roots: Vec<PathBuf> = config
        .search_roots
        .iter()
        .map(|r| expand_tilde(r))
        .filter(|p| p.is_dir())
        .collect();

    for root in roots {
        let mut queue = vec![(root, 0)];
        while !queue.is_empty() {
            let (curr_dir, depth) = queue.remove(0);
            if depth > max_depth {
                continue;
            }
            if let Ok(entries) = fs::read_dir(&curr_dir) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        if !file_type.is_dir() {
                            continue;
                        }

                        let name = entry.file_name().to_string_lossy().into_owned();
                        if name.starts_with('.') || config.exclude_list.contains(&name) {
                            continue;
                        }

                        let path = entry.path();
                        let path_str = path.to_string_lossy().into_owned();
                        let path_lower = path_str.to_lowercase();
                        let keyword_match =
                            intent.kws.is_empty() || is_fuzzy_match(&intent.kws, &name);
                        let tag_match = intent.tags.is_empty()
                            || intent.tags.iter().any(|tag| path_lower.contains(tag));

                        if path_has_project_marker(&path_str, &config.project_markers)
                            && keyword_match
                            && tag_match
                            && !found_paths.contains(&path_str)
                        {
                            found_paths.push(path_str.clone());
                            if found_paths.len() >= limit {
                                return found_paths;
                            }
                        }

                        queue.push((path, depth + 1));
                    }
                }
            }
        }
    }

    found_paths
}

#[cfg(test)]
mod tests {
    use super::{fallback_project_search, match_with_intent, parse_intent};
    use crate::config::Config;
    use crate::db::Entry;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        std::env::temp_dir().join(format!("proton_t_{name}_{nonce}"))
    }

    #[test]
    fn project_intent_accepts_existing_project_markers() {
        let root = unique_temp_dir("project_match");
        let project_dir = root.join("workspace");
        fs::create_dir_all(&project_dir).expect("create project dir");
        fs::write(
            project_dir.join("Cargo.toml"),
            "[package]\nname = \"demo\"\n",
        )
        .expect("write marker");

        let entry = Entry {
            score: 1.0,
            last_access: 1.0,
            is_project: false,
            removed: false,
        };
        let intent = parse_intent(&["project".to_string()]);
        let result = match_with_intent(
            &project_dir.to_string_lossy(),
            &entry,
            &intent,
            &["project".to_string()],
            &["Cargo.toml".to_string()],
            10.0,
        );

        assert!(result.0);

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn fallback_project_search_discovers_project_directories() {
        let root = unique_temp_dir("project_fallback");
        let project_dir = root.join("service-api");
        fs::create_dir_all(&project_dir).expect("create project dir");
        fs::write(
            project_dir.join("Cargo.toml"),
            "[package]\nname = \"demo\"\n",
        )
        .expect("write marker");

        let config = Config {
            search_roots: vec![root.to_string_lossy().into_owned()],
            ..Config::default()
        };
        let intent = parse_intent(&["project".to_string()]);
        let matches = fallback_project_search(&config, &intent, 10);

        assert!(matches.contains(&project_dir.to_string_lossy().into_owned()));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }
}
