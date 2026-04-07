#!/bin/bash
t() {
    if [ $# -eq 0 ]; then
        proton-t list
        return
    fi
    [ "$1" = "-" ] && { cd -; return; }
    [ "$1" = ".." ] && { cd ..; return; }

    RESULT=$(proton-t query "$@")
    if [ -n "$RESULT" ]; then
        [ "$_PT_ECHO" = "1" ] && echo "$RESULT"
        cd "$RESULT"
    else
        echo "proton-t: no match found for '$*'"
        return 1
    fi
}

ti() {
    # If keywords are provided, use them; otherwise show top visited
    RESULT=$(proton-t interactive "$@")
    if [ -n "$RESULT" ]; then
        [ "$_ZO_ECHO" = "1" ] && echo "$RESULT"
        cd "$RESULT"
    fi
}

_proton_t_chpwd() {
    if command -v proton-t >/dev/null 2>&1; then
        proton-t add "$(pwd)"
    fi
}

if [ -n "$BASH_VERSION" ]; then
    [[ ! "$PROMPT_COMMAND" =~ "_proton_t_chpwd" ]] && PROMPT_COMMAND="_proton_t_chpwd; $PROMPT_COMMAND"
elif [ -n "$ZSH_VERSION" ]; then
    autoload -U add-zsh-hook; add-zsh-hook chpwd _proton_t_chpwd; _proton_t_chpwd
fi
