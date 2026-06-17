# Bash completion for its-routing CLI

_its_net_completions() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    opts="--help --version --config -c start-node client-send client-receive status-audit verify-path list-peers time-lock time-unlock time-deny client-export-share client-import-share"

    case "$prev" in
        --config|-c)
            COMPREPLY=( $(compgen -f -- "$cur") )
            return 0
            ;;
        start-node)
            COMPREPLY=( $(compgen -W "--chaff-rate --port --daemonize" -- "$cur") )
            return 0
            ;;
        client-send)
            COMPREPLY=( $(compgen -W "--msg --dest --aeh" -- "$cur") )
            return 0
            ;;
        client-receive)
            COMPREPLY=( $(compgen -W "--source --aeh --unwrap" -- "$cur") )
            return 0
            ;;
        verify-path)
            COMPREPLY=( $(compgen -f -- "$cur") )
            return 0
            ;;
        time-lock)
            COMPREPLY=( $(compgen -W "-f --file -e --epochs -o --out" -- "$cur") )
            return 0
            ;;
        time-unlock)
            COMPREPLY=( $(compgen -W "-p --puzzle -o --out" -- "$cur") )
            return 0
            ;;
        time-deny)
            COMPREPLY=( $(compgen -W "-p --puzzle -d --decoy -o --out" -- "$cur") )
            return 0
            ;;
        *)
            ;;
    esac

    COMPREPLY=( $(compgen -W "$opts" -- "$cur") )
    return 0
}

complete -F _its_net_completions its-routing
