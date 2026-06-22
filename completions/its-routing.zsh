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
                'start-node[Dev-only onion routing daemon]' \
                'client-send[Publish Shannon ITS wire to UES Monocell Pool]' \
                'client-receive[Harvest pool + reconstruct wire]' \
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
                        '-d[The destination Node ID]:node_id:' \
                        '--dest[The destination Node ID]:node_id:' \
                        '--pool[UES Monocell Pool transport]' \
                        '--no-pool[Disable pool default]' \
                        '--aeh[Manual AEH last-resort]' \
                        '--continuous[Enable continuous background decoy chaffing]' \
                        '--ratchet-seed-file[32-byte OTP seed from ITS-KeyManagement]:file:_files' \
                        '--fingerprint-erasure[Optional Gamma CR-NF before send]'
                    ;;
                client-receive)
                    _arguments \
                        '--pool[UES Monocell Pool harvest]' \
                        '--no-pool[Disable pool default]' \
                        '--aeh[Manual AEH scan]' \
                        '--continuous[Epoch-loop receive until wire found]' \
                        '--ratchet-seed-file[32-byte OTP seed from ITS-KeyManagement]:file:_files' \
                        '-o[Output path for received payload]:file:_files' \
                        '--out[Output path for received payload]:file:_files' \
                        '--timeout-secs[Receive timeout in seconds]:seconds:' \
                        '--mailbox-fingerprint[PoolMailbox contact hint]:hex:' \
                        '--mailbox-strict[Reject reconstructions failing wire/OTM gate]'
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
