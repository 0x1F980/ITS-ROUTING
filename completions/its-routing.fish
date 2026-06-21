# Fish completion for its-routing CLI

# Disable file completion by default for commands
complete -c its-routing -f

# Global flags
complete -c its-routing -s c -l config -r -g -d "Specify path to config.toml"
complete -c its-routing -s h -l help -d "Show help menu"
complete -c its-routing -s v -l version -d "Show tool version"

# Commands
complete -c its-routing -n "__fish_use_subcommand" -a start-node -d "Start an active VPS routing node"
complete -c its-routing -n "__fish_use_subcommand" -a client-send -d "Send an Onion-encrypted packet"
complete -c its-routing -n "__fish_use_subcommand" -a client-receive -d "Receive and reconstruct packets"
complete -c its-routing -n "__fish_use_subcommand" -a time-lock -d "Generate a hybrid deniable time-lock puzzle"
complete -c its-routing -n "__fish_use_subcommand" -a time-unlock -d "Solve a time-lock puzzle sequentially"
complete -c its-routing -n "__fish_use_subcommand" -a time-deny -d "Build a decoy puzzle for deniability"
complete -c its-routing -n "__fish_use_subcommand" -a fingerprint-erasure -d "Standalone offline provenance erasure"
complete -c its-routing -n "__fish_use_subcommand" -a client-export-share -d "Export Shamir shares as physical strings"
complete -c its-routing -n "__fish_use_subcommand" -a client-import-share -d "Import Shamir shares from physical strings"

# Subcommand-specific arguments
# start-node
complete -c its-routing -n "__fish_seen_subcommand_from start-node" -s p -l port -r -d "Local port for bare-metal UDP binder"
complete -c its-routing -n "__fish_seen_subcommand_from start-node" -s r -l chaff-rate -r -d "Tick rate for dummy traffic in ms"

# client-send
complete -c its-routing -n "__fish_seen_subcommand_from client-send" -s m -l msg -r -d "The secret message to fragment and encrypt"
complete -c its-routing -n "__fish_seen_subcommand_from client-send" -s f -l file -r -F -d "File payload to send"
complete -c its-routing -n "__fish_seen_subcommand_from client-send" -s d -l dest -r -d "The recipient ID in Z_{2^31-1}"
complete -c its-routing -n "__fish_seen_subcommand_from client-send" -l aeh -d "Enable Ambient Entropy Harvesting mode"
complete -c its-routing -n "__fish_seen_subcommand_from client-send" -l continuous -d "Enable continuous background decoy chaffing"
complete -c its-routing -n "__fish_seen_subcommand_from client-send" -l ratchet-seed-file -r -F -d "32-byte OTP seed from ITS-KeyManagement"

# client-receive
complete -c its-routing -n "__fish_seen_subcommand_from client-receive" -l aeh -d "Use Ambient Entropy Harvesting mode"
complete -c its-routing -n "__fish_seen_subcommand_from client-receive" -l continuous -d "Enable continuous background winnowing"
complete -c its-routing -n "__fish_seen_subcommand_from client-receive" -l ratchet-seed-file -r -F -d "32-byte OTP seed from ITS-KeyManagement"
complete -c its-routing -n "__fish_seen_subcommand_from client-receive" -s o -l out -r -F -d "Output path for received payload"
complete -c its-routing -n "__fish_seen_subcommand_from client-receive" -l timeout-secs -r -d "Receive timeout in seconds"

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
