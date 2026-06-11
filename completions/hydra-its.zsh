#compdef hydra-its

_hydra_its() {
    local line

    _arguments -C \
        '(-c --config)'{-c,--config}'[Path to the config.toml file]:config file:_files' \
        '(-h --help)'{-h,--help}'[Display help message]' \
        '(-v --version)'{-v,--version}'[Print version information]' \
        '1:commands:->cmds' \
        '*::args:->args'

    case $state in
        cmds)
            _values "hydra-its command" \
                'start-node[Start a bare-metal active VPS routing node]' \
                'client-send[Send an Onion-encrypted, fragmented packet]' \
                'client-receive[Receive, reconstruct and verify packets]' \
                'status-audit[Run local telemetry and check for timing anomalies]' \
                'verify-path[Interpolate and verify the homomorphic routing path]' \
                'list-peers[Show current peers in the local finite field routing table]'
            ;;
        args)
            case $line[1] in
                start-node)
                    _arguments \
                        '--chaff-rate[Tick rate in ms for constant-rate chaff]:ms:' \
                        '--port[Listening UDP port]:port:' \
                        '--daemonize[Run routing node as a background daemon]'
                    ;;
                client-send)
                    _arguments \
                        '--msg[The secret payload string to transmit]:message:' \
                        '--dest[The destination Node ID in Z_251]:node_id:' \
                        '--pep[Inject packet into external public entropy stream]'
                    ;;
                client-receive)
                    _arguments \
                        '--source[Source Node ID to filter]:node_id:' \
                        '--pep[Extract message using Passive Entropy Parasitism]' \
                        '--unwrap[Force decryption using local SSS trapdoor]'
                    ;;
                verify-path)
                    _arguments \
                        '--probes[JSON/CSV file containing received probe shares]:file:_files'
                    ;;
            esac
            ;;
    esac
}

_hydra_its "$@"
