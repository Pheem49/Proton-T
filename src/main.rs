mod config;
mod db;
mod search;

use clap::{Parser, Subcommand};
use colored::*;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(
    name = "proton-t",
    version,
    about = "A smarter cd command with frecency and fallback search"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a path to the database
    Add { path: String },
    /// Query for a path
    Query { keywords: Vec<String> },
    /// List paths ranking
    List,
    /// Interactive selection
    Interactive { keywords: Vec<String> },
    /// Remove a directory from the tracking database
    Remove { path: String },
    /// Remove invalid directories from the tracking database
    Clean,
    /// Generate completions for the given keywords
    Complete { keywords: Vec<String> },
    /// Generate shell initialization script
    Init { shell: String },
}

fn print_preview(path: &str, config: &config::Config) {
    if !config.preview_on_jump {
        return;
    }

    if let Ok(entries) = fs::read_dir(path) {
        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    dirs.push(name);
                } else {
                    files.push(name);
                }
            }
        }

        dirs.sort();
        files.sort();

        let total = dirs.len() + files.len();
        if total == 0 {
            return;
        }

        eprintln!("\n🚀 {}", format!("Jumped to: {}", path).cyan().bold());
        eprintln!("📂 {}", format!("Contents ({} items):", total).yellow());

        let mut printed = 0;
        let limit = 7;

        for d in &dirs {
            if printed >= limit {
                break;
            }
            eprintln!("  📁 {}", d.blue());
            printed += 1;
        }

        for f in &files {
            if printed >= limit {
                break;
            }
            eprintln!("  📄 {}", f);
            printed += 1;
        }

        if total > limit {
            eprintln!("  ... and {} more", total - limit);
        }
        eprintln!();
    }
}

fn is_project(path: &str, markers: &[String]) -> bool {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if markers.contains(&name) {
                return true;
            }
        }
    }
    false
}

fn add_path(path_str: String, config: &config::Config) {
    add_path_with_behavior(path_str, config, true);
}

fn add_path_with_behavior(path_str: String, config: &config::Config, revive_removed: bool) {
    let p = Path::new(&path_str);
    if !p.is_dir() {
        return;
    }
    let abs_path = match fs::canonicalize(p) {
        Ok(np) => np.to_string_lossy().into_owned(),
        Err(_) => path_str,
    };

    let parts: Vec<&str> = abs_path.split(std::path::MAIN_SEPARATOR).collect();
    if parts.iter().any(|part| config.exclude_list.contains(*part)) {
        return;
    }

    let mut db = db::load_db();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    let proj_flag = is_project(&abs_path, &config.project_markers);

    if let Some(entry) = db.get_mut(&abs_path) {
        if entry.removed {
            if !revive_removed {
                return;
            }
            entry.removed = false;
        }
        entry.score += 1.0;
        entry.last_access = now;
        entry.is_project = proj_flag;
    } else {
        db.insert(
            abs_path,
            db::Entry {
                score: 1.0,
                last_access: now,
                is_project: proj_flag,
                removed: false,
            },
        );
    }

    db::save_db(db, config.max_entries);
}

fn filter_removed_paths(
    paths: Vec<String>,
    entries: &std::collections::HashMap<String, db::Entry>,
) -> Vec<String> {
    paths
        .into_iter()
        .filter(|path| {
            !entries
                .get(path)
                .map(|entry| entry.removed)
                .unwrap_or(false)
        })
        .collect()
}

fn query_paths(keywords: &[String], config: &config::Config) -> Option<String> {
    if keywords.is_empty() {
        return None;
    }

    let db = db::load_db();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    let intent = search::parse_intent(keywords);

    let full_search = keywords.join(" ");
    if let Ok(p) = fs::canonicalize(PathBuf::from(&full_search)) {
        if p.is_dir() {
            let path = p.to_string_lossy().into_owned();
            if !db.get(&path).map(|entry| entry.removed).unwrap_or(false) {
                return Some(path);
            }
        }
    }

    let mut matches = Vec::new();
    for (path, entry) in &db {
        if entry.removed {
            continue;
        }
        let (matched, score) =
            search::match_with_intent(path, entry, &intent, &config.project_markers, keywords, now);
        if matched {
            matches.push((path.clone(), score));
        }
    }

    if !matches.is_empty() {
        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        return Some(matches[0].0.clone());
    }

    let mut fallbacks = if intent.project {
        let project_matches = search::fallback_project_search(config, &intent, 2);
        if project_matches.is_empty() {
            search::fallback_search(config, keywords, 2)
        } else {
            project_matches
        }
    } else {
        search::fallback_search(config, keywords, 2)
    };
    fallbacks = filter_removed_paths(fallbacks, &db);
    if !fallbacks.is_empty() {
        fallbacks.sort_by(|a, b| {
            let a_base = Path::new(a)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase();
            let b_base = Path::new(b)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase();
            let a_match = keywords.iter().all(|k| a_base.contains(&k.to_lowercase()));
            let b_match = keywords.iter().all(|k| b_base.contains(&k.to_lowercase()));
            b_match.cmp(&a_match)
        });
        let best = fallbacks.remove(0);
        add_path_with_behavior(best.clone(), config, false);
        return Some(best);
    }

    None
}

fn get_all_matches(keywords: &[String], config: &config::Config) -> Vec<String> {
    if keywords.is_empty() {
        return Vec::new();
    }

    let db = db::load_db();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    let intent = search::parse_intent(keywords);

    let mut matches = Vec::new();
    for (path, entry) in &db {
        let parts: Vec<&str> = path.split(std::path::MAIN_SEPARATOR).collect();
        if parts.iter().any(|part| config.exclude_list.contains(*part)) || entry.removed {
            continue;
        }

        let (matched, score) =
            search::match_with_intent(path, entry, &intent, &config.project_markers, keywords, now);
        if matched {
            matches.push((path.clone(), score));
        }
    }

    matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut results: Vec<String> = matches.into_iter().map(|m| m.0).collect();

    let mut fallbacks = if intent.project {
        search::fallback_project_search(config, &intent, 10)
    } else {
        Vec::new()
    };
    fallbacks.extend(search::fallback_search(config, keywords, 10));
    for f in filter_removed_paths(fallbacks, &db) {
        if !results.contains(&f) {
            results.push(f);
        }
    }

    results
}

fn print_section(
    title: &str,
    items: &[(String, f64, db::Entry)],
    shown: &mut std::collections::HashSet<String>,
) {
    let mut count = 0;
    let mut has_title = false;
    for (p, _, _) in items {
        if count >= 3 {
            break;
        }
        if !shown.contains(p) {
            if !has_title {
                println!("\n{}", title.cyan());
                has_title = true;
            }
            println!("  - {}", p.green());
            shown.insert(p.clone());
            count += 1;
        }
    }
}

fn add_menu_section(title: &str, items: &[(String, f64, db::Entry)], matches: &mut Vec<String>) {
    let mut count = 0;
    let mut has_title = false;
    for (p, _, _) in items {
        if count >= 3 {
            break;
        }
        if !matches.contains(p) {
            if !has_title {
                eprintln!("\n{}", title.cyan());
                has_title = true;
            }
            matches.push(p.clone());
            eprintln!("  {}) {}", matches.len(), p.green());
            count += 1;
        }
    }
}

fn list_command(_config: &config::Config) {
    let db = db::load_db();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();

    let all_paths: Vec<_> = db
        .into_iter()
        .map(|(p, e)| {
            let score = db::get_score(&e, now);
            (p, score, e)
        })
        .collect();

    if all_paths.is_empty() {
        return;
    }

    let mut projects: Vec<_> = all_paths
        .iter()
        .filter(|(_, _, e)| e.is_project)
        .cloned()
        .collect();
    projects.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut recent = all_paths.clone();
    recent.sort_by(|a, b| {
        b.2.last_access
            .partial_cmp(&a.2.last_access)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut frequent = all_paths.clone();
    frequent.sort_by(|a, b| {
        b.2.score
            .partial_cmp(&a.2.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut shown = std::collections::HashSet::new();
    print_section("[Suggested Projects]", &projects, &mut shown);
    print_section("[Recent Paths]", &recent, &mut shown);
    print_section("[Frequent Paths]", &frequent, &mut shown);
    println!();
}

fn interactive_command(keywords: Vec<String>, config: &config::Config) {
    let db = db::load_db();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();

    let valid_keywords: Vec<String> = keywords
        .into_iter()
        .filter(|k| !k.trim().is_empty())
        .collect();
    let mut matches = if !valid_keywords.is_empty() {
        get_all_matches(&valid_keywords, config)
    } else {
        Vec::new()
    };

    if matches.is_empty() && valid_keywords.is_empty() && !db.is_empty() {
        let all_paths: Vec<_> = db
            .into_iter()
            .map(|(p, e)| {
                let score = db::get_score(&e, now);
                (p, score, e)
            })
            .collect();

        let mut projects: Vec<_> = all_paths
            .iter()
            .filter(|(_, _, e)| e.is_project)
            .cloned()
            .collect();
        projects.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut recent = all_paths.clone();
        recent.sort_by(|a, b| {
            b.2.last_access
                .partial_cmp(&a.2.last_access)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut frequent = all_paths.clone();
        frequent.sort_by(|a, b| {
            b.2.score
                .partial_cmp(&a.2.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        eprintln!("{}", "Select directory:".yellow());
        add_menu_section("[Suggested Projects]", &projects, &mut matches);
        add_menu_section("[Recent Paths]", &recent, &mut matches);
        add_menu_section("[Frequent Paths]", &frequent, &mut matches);
    } else if !matches.is_empty() {
        eprintln!("{}", "Select directory:".yellow());
        for (i, path) in matches.iter().take(10).enumerate() {
            eprintln!("  {}) {}", i + 1, path.green());
        }
    }

    if matches.is_empty() {
        std::process::exit(1);
    }

    if matches.len() == 1 && !valid_keywords.is_empty() {
        print_preview(&matches[0], config);
        println!("{}", matches[0]);
        add_path(matches[0].clone(), config);
        return;
    }

    let max_choices = std::cmp::min(matches.len(), 10);
    eprint!("\nSelection [1-{}, q to quit]: ", max_choices);
    io::stderr().flush().unwrap();

    let mut choice = String::new();
    if io::stdin().read_line(&mut choice).is_ok() {
        let choice = choice.trim().to_lowercase();
        if choice.is_empty() || choice == "q" {
            std::process::exit(0);
        }
        if let Ok(idx) = choice.parse::<usize>() {
            if idx > 0 && idx <= matches.len() {
                let selected = &matches[idx - 1];
                print_preview(selected, config);
                println!("{}", selected);
                add_path(selected.clone(), config);
                return;
            }
        }
    }
    eprintln!();
    std::process::exit(1);
}

fn main() {
    let cli = Cli::parse();
    let config = config::get_config();

    match cli.command {
        Commands::Add { path } => {
            add_path(path, &config);
        }
        Commands::Query { keywords } => {
            if let Some(res) = query_paths(&keywords, &config) {
                println!("{}", res);
            } else {
                std::process::exit(1);
            }
        }
        Commands::List => {
            list_command(&config);
        }
        Commands::Interactive { keywords } => {
            interactive_command(keywords, &config);
        }
        Commands::Remove { path } => {
            let mut db = db::load_db();
            if let Ok(abs_path) = fs::canonicalize(&path) {
                let p_str = abs_path.to_string_lossy().into_owned();
                let was_project = db
                    .get(&p_str)
                    .map(|entry| entry.is_project)
                    .unwrap_or_else(|| is_project(&p_str, &config.project_markers));
                db.insert(
                    p_str,
                    db::Entry {
                        score: 0.0,
                        last_access: 0.0,
                        is_project: was_project,
                        removed: true,
                    },
                );
                db::save_db(db, config.max_entries);
                println!("{} '{}' from database.", "Removed".green(), path);
                return;
            } else if db.contains_key(&path) {
                if let Some(entry) = db.get_mut(&path) {
                    entry.score = 0.0;
                    entry.last_access = 0.0;
                    entry.removed = true;
                }
                db::save_db(db, config.max_entries);
                println!("{} '{}' from database.", "Removed".green(), path);
                return;
            }
            println!("{} '{}' not found in database.", "Path".yellow(), path);
            std::process::exit(1);
        }
        Commands::Clean => {
            let mut db = db::load_db();
            let mut to_remove = Vec::new();
            for path in db.keys() {
                if !Path::new(path).is_dir() {
                    to_remove.push(path.clone());
                }
            }
            let count = to_remove.len();
            for path in to_remove {
                db.remove(&path);
            }
            if count > 0 {
                db::save_db(db, config.max_entries);
            }
            println!(
                "{} {} invalid paths from the database.",
                "Cleaned".green(),
                count
            );
        }
        Commands::Complete { keywords } => {
            let results = get_all_matches(&keywords, &config);
            let mut seen = std::collections::HashSet::new();
            for path in results {
                if let Some(name) = Path::new(&path).file_name() {
                    let name_str = name.to_string_lossy().to_string();
                    if !seen.contains(&name_str) {
                        println!("{}", name_str);
                        seen.insert(name_str);
                    }
                }
            }
        }
        Commands::Init { shell } => {
            match shell.to_lowercase().as_str() {
                "bash" | "zsh" => {
                    print!("{}", include_str!("../shell_init.sh"));
                }
                "fish" => {
                    print!("{}", include_str!("../init.fish"));
                }
                "powershell" | "pwsh" => {
                    print!("{}", include_str!("../init.ps1"));
                }
                _ => {
                    eprintln!("Error: Unsupported shell '{}'. Supported shells: bash, zsh, fish, powershell", shell);
                    std::process::exit(1);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::filter_removed_paths;
    use crate::db::Entry;
    use std::collections::HashMap;

    #[test]
    fn removed_paths_are_filtered_from_candidates() {
        let entries = HashMap::from([
            (
                "/tmp/skip".to_string(),
                Entry {
                    score: 0.0,
                    last_access: 0.0,
                    is_project: false,
                    removed: true,
                },
            ),
            (
                "/tmp/keep".to_string(),
                Entry {
                    score: 1.0,
                    last_access: 1.0,
                    is_project: false,
                    removed: false,
                },
            ),
        ]);

        let filtered = filter_removed_paths(
            vec!["/tmp/keep".to_string(), "/tmp/skip".to_string()],
            &entries,
        );

        assert_eq!(filtered, vec!["/tmp/keep".to_string()]);
    }
}
