# Proton-T

**The Smarter Way to Navigate.**

Proton-T is a high-performance directory navigation utility designed to help you jump into your projects instantly. It learns your workflow, ranks your favorite folders, and discovers new ones with zero friction.

---

## AI Navigation System

### Project Awareness
Proton-T automatically detects if a folder is a workspace (via `.git`, `package.json`, `Cargo.toml`, etc.) and seamlessly elevates its status.
- Typing `t project` filters the search specifically for developer workspaces.
- Projects are naturally boosted in ranking priority over standard folders.

### Suggestion UI
Don't know where to go? Just type `t` or `ti` with no arguments. Proton-T anticipates your needs by categorizing your paths:
- **[Suggested Projects]**: Top algorithmic picks based on your recent workflow.
- **[Recent Paths]**: Strictly the folders you visited moments ago.
- **[Frequent Paths]**: Your all-time most visited directories.

### Smart Query (Intent Engine)
Go beyond exact character matching. Speak to your terminal:
- `t recent project` -> Computes "What was that project I touched yesterday?" and instantly jumps there.
- `t backend` -> Maps abstract tags like `backend` to folders named `api` or `server`.

---

## Core Capabilities

### Near-Instant Jumps
Skip the `cd` chain. Use short keywords to warp into any directory. Proton-T is written in optimized, lightweight Python for maximum responsiveness and minimal system impact.

### Frecency Engine
Our algorithm calculates the perfect balance between frequency and recency. It gets smarter the more you use it, prioritizing your current active projects while remembering your long-term favorites.

### Fuzzy & Sequence Search
Don't worry about typing the exact name. The smart fuzzy matcher finds your folders even if you skip characters (e.g., `ptn` -> `proton-project`).

---

## Advanced Features

### Smart Fallback Discovery
Never visited a folder before? No problem. Proton-T automatically scans your common workspace roots (Projects, Downloads, etc.) to help you find and bookmark new directories on the fly.

### Interactive Selection (ti)
When paths collide or you want to browse, just use `ti`. A clean numbered menu lets you pick the exact destination when multiple matches are found.

### Smart Blacklist
Say goodbye to junk suggestions. Proton-T automatically filters out noise like `node_modules`, `.git`, `.venv`, and `__pycache__` to keep your navigation clean.

---

## Quick Install

### Universal One-liner
Proton-T works everywhere. Install it in seconds on Linux, macOS, or Windows:

**Linux / macOS (Bash/Zsh)**
```bash
curl -sSfL https://raw.githubusercontent.com/Pheem49/Proton-T/main/install.sh | sh
```

**Windows (PowerShell)**
```powershell
iex (Invoke-RestMethod https://raw.githubusercontent.com/Pheem49/Proton-T/main/install.ps1)
```
> [!NOTE]
> On Windows, you might need to run: `Set-ExecutionPolicy RemoteSigned -Scope CurrentUser`

---

## Usage

| Command | Action |
| :--- | :--- |
| `t <query>` | Jump to the best match |
| `ti <query>` | Open interactive selection |
| `t -` | Go back to the previous directory |
| `t ..` | Go up one level |
| `proton-t list` | View current directory rankings |

---

## Configuration
- `_PT_ECHO=1`: Print the target path before jumping.
- Database: `~/.proton_t_db.json`
