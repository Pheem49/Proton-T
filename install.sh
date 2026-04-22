#!/bin/bash
set -e

REPO_URL="https://github.com/Pheem49/Proton-T.git"
INSTALL_DIR="$HOME/.proton-t"

# Bootstrap: If not in the project folder, clone it to ~/.proton-t
if [ ! -f "Cargo.toml" ]; then
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
# Install as Rust Binary
if ! command -v cargo >/dev/null 2>&1; then
    # Try sourcing cargo env in case it's installed but not in PATH
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "Rust is not installed. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi
cargo install --path "$PROJECT_DIR"

# Integration in .bashrc
if [ -f "$BASHRC" ]; then
    if ! grep -qF "proton-t init bash" "$BASHRC"; then
        printf "\n# Proton-T Integration\neval \"\$(proton-t init bash)\"\n" >> "$BASHRC"
    fi
fi

# Integration in .zshrc
ZSHRC="$HOME/.zshrc"
if [ -f "$ZSHRC" ]; then
    if ! grep -qF "proton-t init zsh" "$ZSHRC"; then
        printf "\n# Proton-T Integration\neval \"\$(proton-t init zsh)\"\n" >> "$ZSHRC"
    fi
fi

# Integration in config.fish
FISH_CONFIG="$HOME/.config/fish/config.fish"
if command -v fish >/dev/null 2>&1; then
    if [ ! -f "$FISH_CONFIG" ]; then
        mkdir -p "$(dirname "$FISH_CONFIG")"
        touch "$FISH_CONFIG"
    fi
    if ! grep -qF "proton-t init fish" "$FISH_CONFIG"; then
        printf "\n# Proton-T Integration\nproton-t init fish | source\n" >> "$FISH_CONFIG"
    fi
fi

echo "Done! Please run: source ~/.bashrc (or restart your shell)"
