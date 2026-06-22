# Fish completion for its-routing CLI

complete -c its-routing -f

# Global flags
complete -c its-routing -s c -l config -r -g -d "Specify path to config.toml"
complete -c its-routing -s h -l help -d "Show help menu"
complete -c its-routing -s v -l version -d "Show tool version"

# Commands
complete -c its-routing -n "__fish_use_subcommand" -a start-node -d "Dev-only onion routing daemon"
complete -c its-routing -n "__fish_use_subcommand" -a client-send -d "Publish Shannon ITS wire to pool"
complete -c its-routing -n "__fish_use_subcommand" -a client-receive -d "Harvest pool and reconstruct wire"
complete -c its-routing -n "__fish_use_subcommand" -a time-lock -d "Generate a hybrid deniable time-lock puzzle"
complete -c its-routing -n "__fish_use_subcommand" -a time-unlock -d "Solve a time-lock puzzle sequentially"
complete -c its-routing -n "__fish_use_subcommand" -a time-deny -d "Build a decoy puzzle for deniability"
complete -c its-routing -n "__fish_use_subcommand" -a fingerprint-erasure -d "Standalone offline provenance erasure"
complete -c its-routing -n "__fish_use_subcommand" -a client-export-share -d "Export Shamir shares as physical strings"
complete -c its-routing -n "__fish_use_subcommand" -a client-import-share -d "Import Shamir shares from physical strings"

# start-node (dev-only)
complete -c its-routing -n "__fish_seen_subcommand_from start-node" -s p -l port -r -d "Local port for bare-metal UDP binder"
complete -c its-routing -n "__fish_seen_subcommand_from start-node" -s r -l chaff-rate -r -d "Tick rate for dummy traffic in ms"

# client-send (--pool, --no-pool, --aeh, --ratchet-seed-file)
complete -c its-routing -n "__fish_seen_subcommand_from client-send" -s m -l msg -r -d "The secret message to fragment and encrypt"
complete -c its-routing -n "__fish_seen_subcommand_from client-send" -s f -l file -r -F -d "File payload to send"
complete -c its-routing -n "__fish_seen_subcommand_from client-send" -s d -l dest -r -d "The recipient ID"
complete -c its-routing -n "__fish_seen_subcommand_from client-send" -l pool -d "UES Monocell Pool transport (default when transport_mode=pool)"
complete -c its-routing -n "__fish_seen_subcommand_from client-send" -l no-pool -d "Disable pool default"
complete -c its-routing -n "__fish_seen_subcommand_from client-send" -l aeh -d "Manual AEH last-resort"
complete -c its-routing -n "__fish_seen_subcommand_from client-send" -l continuous -d "Enable continuous background decoy chaffing"
complete -c its-routing -n "__fish_seen_subcommand_from client-send" -l ratchet-seed-file -r -F -d "32-byte OTP seed from ITS-KeyManagement"
complete -c its-routing -n "__fish_seen_subcommand_from client-send" -l fingerprint-erasure -d "Optional Gamma CR-NF before send"
complete -c its-routing -n "__fish_seen_subcommand_from client-send" -l mailbox-fingerprint -r -d "PoolMailbox contact hint"

# client-receive
complete -c its-routing -n "__fish_seen_subcommand_from client-receive" -l pool -d "UES Monocell Pool harvest"
complete -c its-routing -n "__fish_seen_subcommand_from client-receive" -l no-pool -d "Disable pool default"
complete -c its-routing -n "__fish_seen_subcommand_from client-receive" -l aeh -d "Manual AEH scan"
complete -c its-routing -n "__fish_seen_subcommand_from client-receive" -l continuous -d "Epoch-loop receive until wire found"
complete -c its-routing -n "__fish_seen_subcommand_from client-receive" -l ratchet-seed-file -r -F -d "32-byte OTP seed from ITS-KeyManagement"
complete -c its-routing -n "__fish_seen_subcommand_from client-receive" -s o -l out -r -F -d "Output path for received payload"
complete -c its-routing -n "__fish_seen_subcommand_from client-receive" -l timeout-secs -r -d "Receive timeout in seconds"
complete -c its-routing -n "__fish_seen_subcommand_from client-receive" -l mailbox-fingerprint -r -d "PoolMailbox contact hint"
complete -c its-routing -n "__fish_seen_subcommand_from client-receive" -l mailbox-strict -d "Reject reconstructions failing wire/OTM gate"

# time-lock
complete -c its-routing -n "__fish_seen_subcommand_from time-lock" -s f -l file -r -F -d "Document to lock"
complete -c its-routing -n "__fish_seen_subcommand_from time-lock" -s e -l epochs -r -d "Sequential squaring rounds (default 1000)"
complete -c its-routing -n "__fish_seen_subcommand_from time-lock" -s o -l out -r -F -d "Output .its puzzle file"

# time-unlock
complete -c its-routing -n "__fish_seen_subcommand_from time-unlock" -s p -l puzzle -r -F -d "Input .its puzzle file"
complete -c its-routing -n "__fish_seen_subcommand_from time-unlock" -s o -l out -r -F -d "Decrypted output file"

# time-deny
complete -c its-routing -n "__fish_seen_subcommand_from time-deny" -s p -l puzzle -r -F -d "Input .its puzzle file"
complete -c its-routing -n "__fish_seen_subcommand_from time-deny" -s d -l decoy -r -d "Decoy message of equal length"
complete -c its-routing -n "__fish_seen_subcommand_from time-deny" -s o -l out -r -F -d "Alternative .its puzzle file"
