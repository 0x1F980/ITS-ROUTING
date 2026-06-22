# Bash completion for its-routing CLI

_its_routing_completions() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    opts="--help --version --config -c start-node client-send client-receive time-lock time-unlock time-deny fingerprint-erasure client-export-share client-import-share"

    case "$prev" in
        --config|-c)
            COMPREPLY=( $(compgen -f -- "$cur") )
            return 0
            ;;
        start-node)
            COMPREPLY=( $(compgen -W "-p --port -r --chaff-rate" -- "$cur") )
            return 0
            ;;
        client-send)
            COMPREPLY=( $(compgen -W "-m --msg -f --file -d --dest --pool --no-pool --aeh --continuous --ratchet-seed-file --fingerprint-erasure --fe-strict --fe-permissive" -- "$cur") )
            return 0
            ;;
        client-receive)
            COMPREPLY=( $(compgen -W "--pool --no-pool --aeh --continuous --ratchet-seed-file -o --out --timeout-secs --mailbox-fingerprint --mailbox-strict" -- "$cur") )
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

complete -F _its_routing_completions its-routing
