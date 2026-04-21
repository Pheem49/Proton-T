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
        [ "$_PT_ECHO" = "1" ] && echo "$RESULT"
        cd "$RESULT"
    fi
}

_proton_t_chpwd() {
    if command -v proton-t >/dev/null 2>&1; then
        proton-t add "$(pwd)"
    fi
}

_proton_t_complete() {
    local args=("${COMP_WORDS[@]:1:COMP_CWORD}")
    local cur=${COMP_WORDS[COMP_CWORD]}
    local completions

    if [ ${#args[@]} -eq 0 ]; then
        completions=$(proton-t complete "$cur")
    else
        completions=$(proton-t complete "${args[@]}")
    fi

    mapfile -t COMPREPLY < <(printf '%s\n' "$completions" | sed '/^$/d')
}

if [ -n "$BASH_VERSION" ]; then
    cd() {
        builtin cd "$@" || return
        _proton_t_chpwd
    }

    _proton_t_chpwd
    complete -F _proton_t_complete t
elif [ -n "$ZSH_VERSION" ]; then
    autoload -U add-zsh-hook; add-zsh-hook chpwd _proton_t_chpwd; _proton_t_chpwd
    
    _proton_t_zsh_complete() {
        local -a matches
        matches=(${(f)"$(proton-t complete "${words[@]:1}")"})
        _describe 'directories' matches
    }
    compdef _proton_t_zsh_complete t
fi
