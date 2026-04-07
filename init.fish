function t --description 'Proton-T: Smart directory jump'
    if not set -q argv[1]
        proton-t list
        return
    end

    if test "$argv[1]" = "-"
        prevd
        return
    end

    if test "$argv[1]" = ".."
        builtin cd ..
        return
    end

    if test "$argv[1]" = "."
        return
    end

    set -l result (proton-t query $argv)
    if test -n "$result"
        if test "$_PT_ECHO" = "1"
            echo $result
        end
        builtin cd $result
    else
        echo "proton-t: no match found for '$argv'"
    end
end

function ti --description 'Proton-T: Interactive directory selection'
    set -l result (proton-t interactive $argv)
    if test -n "$result"
        if test "$_PT_ECHO" = "1"
            echo $result
        end
        builtin cd $result
    end
end

function __proton_t_add --on-variable PWD --description 'Proton-T: Track directory changes'
    set -l current_dir (pwd)
    proton-t add $current_dir >/dev/null 2>&1
end
