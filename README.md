# Proton-T

A smarter `cd` command. It tracks your directory usage and allows you to jump to your most frequent and recent directories using short keywords.

## Features

- **Frecency Ranking**: Prioritizes directories you visit often and recently.
- **Smart Fallback**: Automatically searches common folders (Downloads, Projects, etc.) if a match isn't in history.
- **Interactive Mode**: Use `ti` to choose from multiple matches when a query is ambiguous.
- **Natural Pathing**: Supports direct paths, `t -` for back, and `t ..` for up.
- **Zero-Config**: Single script installation for Bash and Zsh.

## Quick Install

### Linux / macOS (Bash/Zsh)
```bash
curl -sSfL https://raw.githubusercontent.com/Pheem49/Proton-T/main/install.sh | sh
```

### Windows (PowerShell)
```powershell
iex (Invoke-RestMethod https://raw.githubusercontent.com/Pheem49/Proton-T/main/install.ps1)
```
> [!NOTE]
> If scripts are blocked, run: `Set-ExecutionPolicy RemoteSigned -Scope CurrentUser`

## Manual Installation

Alternatively, you can clone the repository manually:

```bash
git clone https://github.com/Pheem49/Proton-T.git
cd Proton-T
chmod +x install.sh
./install.sh
source ~/.bashrc
```

## Usage

### Basic Jumping
Jump to a directory by typing part of its name:
```bash
t my-project
```

### Interactive Selection
Choose from a list of possible matches:
```bash
ti flask
```

### Special Navigations
- Go back to previous directory: `t -`
- Go up one level: `t ..`

### List History
Show all tracked directories and their scores:
```bash
proton-t list
```

## Configuration

- `_PT_ECHO=1`: Set this environment variable to print the target path before jumping.
- Database is stored at: `~/.proton_t_db.json`
