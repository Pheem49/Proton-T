import os, time, math, json

DB_FILE = os.path.expanduser("~/.proton_t_db.json")
MAX_ENTRIES = 1000
EXCLUDE_LIST = {
    'node_modules', '.git', '__pycache__', '.venv', '.next', '.pytest_cache', '.casbin'
}

def load_db():
    if not os.path.exists(DB_FILE): return {}
    try:
        with open(DB_FILE, 'r') as f: return json.load(f)
    except: return {}

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
    if path in db:
        db[path]['score'] += 1
        db[path]['last_access'] = now
    else:
        db[path] = {'score': 1, 'last_access': now}
    save_db(db)

def get_score(entry, now):
    score, last_access = entry['score'], entry['last_access']
    decay = math.exp(-(now - last_access) / 604800) 
    return score * decay

SEARCH_ROOTS = [
    os.path.expanduser("~"),
    os.path.expanduser("~/Downloads"),
    os.path.expanduser("~/Documents"),
    os.path.expanduser("~/vscode/Project"),
    os.path.expanduser("~/Desktop")
]
# Clean and unique roots, normalized for current OS
SEARCH_ROOTS = sorted(list(set([os.path.normpath(r) for r in SEARCH_ROOTS if os.path.isdir(r)])))

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

    for path, entry in db.items():
        if is_fuzzy_match(keywords, path):
            score = get_score(entry, now)
            # Boost exact substring matches in basename
            basename = os.path.basename(path).lower()
            if all(k.lower() in basename for k in keywords):
                score *= 10
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

    for path, entry in db.items():
        if is_fuzzy_match(keywords, path):
            if is_excluded(path): continue
            score = get_score(entry, now)
            basename = os.path.basename(path).lower()
            if all(k.lower() in basename for k in keywords):
                score *= 10
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
    return sorted([(p, get_score(e, now), e['score']) for p, e in db.items()], key=lambda x: x[1], reverse=True)

def get_directory_preview(path, limit=20):
    """Return a list of files and directories in the path for preview."""
    try:
        entries = []
        with os.scandir(path) as it:
            for entry in it:
                prefix = "DIR  " if entry.is_dir() else "FILE "
                name = entry.name
                if name.startswith('.'): continue
                if name in EXCLUDE_LIST: continue
                entries.append(f"{prefix} {name}")
        
        # Sort so directories come first, then files
        entries.sort()
        return entries[:limit]
    except Exception as e:
        return [f"Error reading directory: {str(e)}"]

def list_current_dir_with_scores():
    """List entries in current directory with their frecency scores."""
    db, now = load_db(), time.time()
    results = []
    try:
        with os.scandir(os.getcwd()) as it:
            for entry in it:
                path = os.path.normpath(os.path.abspath(entry.path))
                score = 0
                is_tracked = False
                if path in db:
                    score = get_score(db[path], now)
                    is_tracked = True
                
                results.append({
                    'name': entry.name,
                    'is_dir': entry.is_dir(),
                    'score': score,
                    'is_tracked': is_tracked
                })
        # Sort: directories first, then by name
        results.sort(key=lambda x: (not x['is_dir'], x['name'].lower()))
        return results
    except Exception as e:
        return []
