#!/bin/bash
set -e

REPO_URL="https://github.com/Pheem49/Proton-T.git"
INSTALL_DIR="$HOME/.proton-t"

# Bootstrap: If not in the project folder, clone it to ~/.proton-t
if [ ! -f "pyproject.toml" ]; then
    if ! command -v git >/dev/null 2>&1; then
        echo "Error: git is not installed."
        exit 1
    fi
    echo "Downloading Proton-T..."
    if [ -d "$INSTALL_DIR" ]; then
        cd "$INSTALL_DIR" && git pull
    else
        git clone "$REPO_URL" "$INSTALL_DIR"
        cd "$INSTALL_DIR"
    fi
fi

PROJECT_DIR=$(pwd)
SHELL_INIT="$PROJECT_DIR/shell_init.sh"
BASHRC="$HOME/.bashrc"

echo "Installing Proton-T..."
# Install as Python Package
pip install "$PROJECT_DIR" --user --break-system-packages 2>/dev/null || pip install "$PROJECT_DIR" --user

# Integration in .bashrc (Avoid duplicates)
if ! grep -qF "source $SHELL_INIT" "$BASHRC"; then
    printf "\n# Proton-T Integration\nsource %s\n" "$SHELL_INIT" >> "$BASHRC"
fi

# Integration in config.fish
FISH_CONFIG="$HOME/.config/fish/config.fish"
FISH_INIT="$PROJECT_DIR/init.fish"
if command -v fish >/dev/null 2>&1; then
    if [ ! -f "$FISH_CONFIG" ]; then
        mkdir -p "$(dirname "$FISH_CONFIG")"
        touch "$FISH_CONFIG"
    fi
    if ! grep -qF "source $FISH_INIT" "$FISH_CONFIG"; then
        printf "\n# Proton-T Integration\nsource %s\n" "$FISH_INIT" >> "$FISH_CONFIG"
    fi
fi

echo "Done! Please run: source ~/.bashrc (or restart your shell)"
