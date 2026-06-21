#compdef its-routing

_its_net() {
    local line

    _arguments -C \
        '(-c --config)'{-c,--config}'[Path to the config.toml file]:config file:_files' \
        '(-h --help)'{-h,--help}'[Display help message]' \
        '(-v --version)'{-v,--version}'[Print version information]' \
        '1:commands:->cmds' \
        '*::args:->args'

    case $state in
        cmds)
            _values "its-routing command" \
                'start-node[Start a bare-metal active VPS routing node]' \
                'client-send[Send an Onion-encrypted, fragmented packet]' \
                'client-receive[Receive, reconstruct and verify packets]' \
                'time-lock[Generate a hybrid deniable time-lock puzzle over a file]' \
                'time-unlock[Sequentially solve and decrypt a .its puzzle]' \
                'time-deny[Build an alternative decoy puzzle for deniability]' \
                'fingerprint-erasure[Standalone offline provenance erasure]' \
                'client-export-share[Export Shamir shares as physical strings]' \
                'client-import-share[Import Shamir shares from physical strings]'
            ;;
        args)
            case $line[1] in
                start-node)
                    _arguments \
                        '-p[Listening UDP port]:port:' \
                        '--port[Listening UDP port]:port:' \
                        '-r[Tick rate in ms for constant-rate chaff]:ms:' \
                        '--chaff-rate[Tick rate in ms for constant-rate chaff]:ms:'
                    ;;
                client-send)
                    _arguments \
                        '-m[The secret payload string to transmit]:message:' \
                        '--msg[The secret payload string to transmit]:message:' \
                        '-f[File payload to send]:file:_files' \
                        '--file[File payload to send]:file:_files' \
                        '-d[The destination Node ID in Z_{2^31-1}]:node_id:' \
                        '--dest[The destination Node ID in Z_{2^31-1}]:node_id:' \
                        '--aeh[Inject packet into external public entropy stream]' \
                        '--continuous[Enable continuous background decoy chaffing]' \
                        '--ratchet-seed-file[32-byte OTP seed from ITS-KeyManagement]:file:_files'
                    ;;
                client-receive)
                    _arguments \
                        '--aeh[Extract message using Ambient Entropy Harvesting]' \
                        '--continuous[Enable continuous background winnowing]' \
                        '--ratchet-seed-file[32-byte OTP seed from ITS-KeyManagement]:file:_files' \
                        '-o[Output path for received payload]:file:_files' \
                        '--out[Output path for received payload]:file:_files' \
                        '--timeout-secs[Receive timeout in seconds]:seconds:'
                    ;;
                time-lock)
                    _arguments \
                        '(-f --file)'{-f,--file}'[Document to lock]:file:_files' \
                        '(-e --epochs)'{-e,--epochs}'[Sequential squaring rounds]:count:' \
                        '(-o --out)'{-o,--out}'[Output .its puzzle file]:file:_files'
                    ;;
                time-unlock)
                    _arguments \
                        '(-p --puzzle)'{-p,--puzzle}'[Input .its puzzle file]:file:_files' \
                        '(-o --out)'{-o,--out}'[Decrypted output file]:file:_files'
                    ;;
                time-deny)
                    _arguments \
                        '(-p --puzzle)'{-p,--puzzle}'[Input .its puzzle file]:file:_files' \
                        '(-d --decoy)'{-d,--decoy}'[Decoy message of equal length]:text:' \
                        '(-o --out)'{-o,--out}'[Alternative .its puzzle file]:file:_files'
                    ;;
            esac
            ;;
    esac
}

_its_net "$@"
