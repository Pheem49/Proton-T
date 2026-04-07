#!/bin/bash
set -e
PROJECT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
SHELL_INIT="$PROJECT_DIR/shell_init.sh"
BASHRC="$HOME/.bashrc"
echo "Installing Proton-T..."
# Install as Python Package
pip install "$PROJECT_DIR" --user --break-system-packages 2>/dev/null || pip install "$PROJECT_DIR" --user
# Integration in .bashrc
if ! grep -qF "source $SHELL_INIT" "$BASHRC"; then
    echo -e "\n# Proton-T Integration\nsource $SHELL_INIT" >> "$BASHRC"
fi
echo "Done! Please run: source ~/.bashrc"
