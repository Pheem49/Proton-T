import os, time, math, json

DB_FILE = os.path.expanduser("~/.proton_t_db.json")
CONFIG_DIR = os.path.expanduser("~/.config/proton-t")
CONFIG_FILE = os.path.join(CONFIG_DIR, "config.json")

DEFAULT_CONFIG = {
    "search_roots": [
        "~",
        "~/Downloads",
        "~/Documents",
        "~/Desktop"
    ],
    "max_entries": 1000,
    "exclude_list": [
        "node_modules", ".git", "__pycache__", ".venv", ".next", ".pytest_cache", ".casbin"
    ],
    "project_markers": [
        ".git", "package.json", "requirements.txt", "Cargo.toml", "go.mod", "pom.xml", "build.gradle"
    ]
}

def load_config():
    if not os.path.exists(CONFIG_DIR):
        try: os.makedirs(CONFIG_DIR, exist_ok=True)
        except OSError: pass
    
    if not os.path.exists(CONFIG_FILE):
        try:
            with open(CONFIG_FILE, 'w') as f:
                json.dump(DEFAULT_CONFIG, f, indent=2)
        except OSError: pass
        return DEFAULT_CONFIG
        
    try:
        with open(CONFIG_FILE, 'r') as f:
            user_config = json.load(f)
            config = DEFAULT_CONFIG.copy()
            config.update(user_config)
            return config
    except (json.JSONDecodeError, OSError):
        return DEFAULT_CONFIG

config = load_config()
MAX_ENTRIES = config.get("max_entries", 1000)
EXCLUDE_LIST = set(config.get("exclude_list", DEFAULT_CONFIG["exclude_list"]))
PROJECT_MARKERS = config.get("project_markers", DEFAULT_CONFIG["project_markers"])

TAG_MAP = {
    "backend": ["api", "server", "node", "backend", "go", "java"],
    "frontend": ["ui", "react", "web", "frontend", "client", "next", "vue"],
}

def is_project(path):
    try:
        with os.scandir(path) as it:
            for entry in it:
                if entry.name in PROJECT_MARKERS:
                    return True
    except (OSError, PermissionError):
        pass
    return False

def load_db():
    if not os.path.exists(DB_FILE): return {}
    try:
        with open(DB_FILE, 'r') as f: return json.load(f)
    except (json.JSONDecodeError, OSError): return {}

def save_db(data):
    if len(data) > MAX_ENTRIES:
        now = time.time()
        sorted_items = sorted(data.items(), key=lambda x: get_score(x[1], now), reverse=True)
        data = dict(sorted_items[:MAX_ENTRIES])
    with open(DB_FILE, 'w') as f: json.dump(data, f, indent=2)

def is_excluded(path):
    parts = os.path.normpath(path).split(os.sep)
    return any(p in EXCLUDE_LIST for p in parts)

def is_fuzzy_match(keywords, path):
    """Check if all keywords appear as fuzzy or substring matches in the path."""
    path_lower = path.lower()
    for kw in keywords:
        kw_lower = kw.lower()
        # Fast substring check first
        if kw_lower in path_lower: continue
        # Then character-to-character sequence check (the 'fuzzy' part)
        it = iter(path_lower)
        if not all(c in it for c in kw_lower): return False
    return True

def add_path(path):
    path = os.path.normpath(os.path.abspath(path))
    if not os.path.isdir(path) or is_excluded(path): return
    db, now = load_db(), time.time()
    
    project_flag = is_project(path)
    
    if path in db:
        db[path]['score'] += 1
        db[path]['last_access'] = now
        db[path]['is_project'] = project_flag
    else:
        db[path] = {'score': 1, 'last_access': now, 'is_project': project_flag}
    save_db(db)

def get_score(entry, now):
    score, last_access = entry['score'], entry['last_access']
    decay = math.exp(-(now - last_access) / 604800) 
    final_score = score * decay
    if entry.get('is_project'):
        final_score *= 1.2
    return final_score

def _init_search_roots():
    roots = []
    for r in config.get("search_roots", DEFAULT_CONFIG["search_roots"]):
        path = os.path.expanduser(r)
        if os.path.isdir(path):
            roots.append(os.path.normpath(path))
    return sorted(list(set(roots)))

SEARCH_ROOTS = _init_search_roots()

def fallback_search(keywords, max_depth=2, limit=10):
    """Scan common folders and return ALL matching directories."""
    found_paths = []
    for root in SEARCH_ROOTS:
        queue = [(root, 0)]
        while queue:
            curr_dir, depth = queue.pop(0)
            if depth > max_depth: continue
            try:
                with os.scandir(curr_dir) as it:
                    for entry in it:
                        if entry.is_dir():
                            if entry.name.startswith('.') or entry.name in EXCLUDE_LIST: continue
                            if is_fuzzy_match(keywords, entry.name):
                                path = os.path.normpath(entry.path)
                                if path not in found_paths:
                                    found_paths.append(path)
                                    if len(found_paths) >= limit: return found_paths
                            queue.append((os.path.normpath(entry.path), depth + 1))
            except (PermissionError, FileNotFoundError):
                continue
    return found_paths

def parse_intent(keywords):
    kws = [k.lower() for k in keywords]
    intent = {'recent': False, 'project': False, 'kws': [], 'tags': []}
    
    for kw in kws:
        if kw in ['last', 'recent', 'latest', 'today']: intent['recent'] = True
        elif kw in ['project', 'app', 'proj']: intent['project'] = True
        else:
            mapped = False
            for tag, syns in TAG_MAP.items():
                if kw in syns or kw == tag:
                    intent['tags'].extend(syns)
                    mapped = True
            if not mapped: intent['kws'].append(kw)
    return intent

def match_with_intent(path, entry, intent, keywords, now):
    # 1. Base fuzzy matching on remaining keywords
    if intent['kws'] and not is_fuzzy_match(intent['kws'], path):
        # Fallback: maybe they literally meant 'lastProject'
        if not is_fuzzy_match(keywords, path):
            return False, 0
            
    # 2. Tag matching
    if intent['tags']:
        if not any(t in path.lower() for t in intent['tags']):
            return False, 0
            
    # 3. Project Filter
    is_proj = entry.get('is_project', False)
    if intent['project'] and not is_proj:
        # Strict filter out if intent was project but this isn't
        # Unless it literally matches the string e.g. folder named `my-project`
        if 'project' not in path.lower():
            return False, 0
            
    score = get_score(entry, now)
    
    # 4. Keyword boosting
    basename = os.path.basename(path).lower()
    if intent['kws'] and all(k.lower() in basename for k in intent['kws']):
        score *= 10
        
    # 5. Recency boost
    if intent['recent']:
        time_diff = max(1, now - entry['last_access'])
        score = (1.0 / time_diff) * 1e9  # Massive override for recent
        
    return True, score

def query_paths(keywords):
    if not keywords: return None
    
    # 1. Direct path check
    full_search = " ".join(keywords)
    potential_path = os.path.normpath(os.path.abspath(os.path.expanduser(full_search)))
    if os.path.isdir(potential_path):
        return potential_path

    # 2. Database search with fuzzy matching
    db, now = load_db(), time.time()
    matches = []
    
    intent = parse_intent(keywords)

    for path, entry in db.items():
        matched, score = match_with_intent(path, entry, intent, keywords, now)
        if matched:
            matches.append((path, score))
    
    if matches:
        matches.sort(key=lambda x: x[1], reverse=True)
        return matches[0][0]

    # 3. Fallback search (Take first match for direct query)
    found = fallback_search(keywords, limit=2) # Check a couple to be safe
    if found:
        # Filter for the best match using basename logic
        found.sort(key=lambda p: all(k.lower() in os.path.basename(p).lower() for k in keywords), reverse=True)
        add_path(found[0])
        return found[0]
        
    return None

def get_all_matches(keywords):
    if not keywords: return []
    
    db, now = load_db(), time.time()
    matches = []
    
    intent = parse_intent(keywords)

    for path, entry in db.items():
        if is_excluded(path): continue
        matched, score = match_with_intent(path, entry, intent, keywords, now)
        if matched:
            matches.append((path, score))
    
    matches.sort(key=lambda x: x[1], reverse=True)
    results = [m[0] for m in matches]
    
    # Also add ALL fallback hits
    fallbacks = fallback_search(keywords, limit=10)
    for f in fallbacks:
        if f not in results:
            results.append(f)
        
    return results

def list_paths():
    db, now = load_db(), time.time()
    return [(p, get_score(e, now), e) for p, e in db.items()]

def remove_path(path):
    path = os.path.normpath(os.path.abspath(path))
    db = load_db()
    if path in db:
        del db[path]
        save_db(db)
        return True
    return False

def clean_db():
    db = load_db()
    to_remove = []
    for path in db:
        if not os.path.isdir(path):
            to_remove.append(path)
    for path in to_remove:
        del db[path]
    if to_remove:
        save_db(db)
    return len(to_remove)

