import os, time, math, json

DB_FILE = os.path.expanduser("~/.proton_t_db.json")
MAX_ENTRIES = 1000

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

def add_path(path):
    path = os.path.abspath(path)
    if not os.path.isdir(path): return
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
# Clean and unique roots
SEARCH_ROOTS = sorted(list(set([r for r in SEARCH_ROOTS if os.path.isdir(r)])))

def fallback_search(keywords, max_depth=2, limit=10):
    """Scan common folders and return ALL matching directories."""
    keywords_lower = [k.lower() for k in keywords]
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
                            if entry.name.startswith('.') or entry.name == "__pycache__": continue
                            if all(k in entry.name.lower() for k in keywords_lower):
                                if entry.path not in found_paths:
                                    found_paths.append(entry.path)
                                    if len(found_paths) >= limit: return found_paths
                            queue.append((entry.path, depth + 1))
            except (PermissionError, FileNotFoundError):
                continue
    return found_paths

def query_paths(keywords):
    if not keywords: return None
    
    # 1. Direct path check
    full_search = " ".join(keywords)
    potential_path = os.path.abspath(os.path.expanduser(full_search))
    if os.path.isdir(potential_path):
        return potential_path

    # 2. Database search
    db, now = load_db(), time.time()
    matches = []
    keywords_lower = [k.lower() for k in keywords]

    for path, entry in db.items():
        if all(k in path.lower() for k in keywords_lower):
            score = get_score(entry, now)
            if any(k == os.path.basename(path).lower() for k in keywords_lower):
                score *= 10
            matches.append((path, score))
    
    if matches:
        matches.sort(key=lambda x: x[1], reverse=True)
        return matches[0][0]

    # 3. Fallback search (Take first match for direct query)
    found = fallback_search(keywords, limit=1)
    if found:
        add_path(found[0])
        return found[0]
        
    return None

def get_all_matches(keywords):
    if not keywords: return []
    
    db, now = load_db(), time.time()
    matches = []
    keywords_lower = [k.lower() for k in keywords]

    for path, entry in db.items():
        if all(k in path.lower() for k in keywords_lower):
            score = get_score(entry, now)
            if any(k == os.path.basename(path).lower() for k in keywords_lower):
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
