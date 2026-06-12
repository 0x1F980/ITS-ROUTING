# Fish completion for its-net CLI

# Disable file completion by default for commands
complete -c its-net -f

# Global flags
complete -c its-net -s c -l config -r -g -d "Specify path to config.toml"
complete -c its-net -s h -l help -d "Show help menu"
complete -c its-net -s v -l version -d "Show tool version"

# Commands
complete -c its-net -n "__fish_use_subcommand" -a start-node -d "Start an active VPS routing node"
complete -c its-net -n "__fish_use_subcommand" -a client-send -d "Send an Onion-encrypted packet"
complete -c its-net -n "__fish_use_subcommand" -a client-receive -d "Receive and reconstruct packets"
complete -c its-net -n "__fish_use_subcommand" -a status-audit -d "Run diagnostic telemetry"
complete -c its-net -n "__fish_use_subcommand" -a verify-path -d "Verify homomorphic routing path"
complete -c its-net -n "__fish_use_subcommand" -a list-peers -d "Show current finite field routing table"

# Subcommand-specific arguments
# start-node
complete -c its-net -n "__fish_seen_subcommand_from start-node" -l chaff-rate -r -d "Tick rate for dummy traffic in ms"
complete -c its-net -n "__fish_seen_subcommand_from start-node" -l port -r -d "Local port for bare-metal UDP binder"
complete -c its-net -n "__fish_seen_subcommand_from start-node" -l daemonize -d "Run routing node in the background"

# client-send
complete -c its-net -n "__fish_seen_subcommand_from client-send" -l msg -r -d "The secret message to fragment and encrypt"
complete -c its-net -n "__fish_seen_subcommand_from client-send" -l dest -r -d "The recipient ID in Z_{2^31-1}"
complete -c its-net -n "__fish_seen_subcommand_from client-send" -l aeh -d "Enable Ambient Entropy Harvesting mode"

# client-receive
complete -c its-net -n "__fish_seen_subcommand_from client-receive" -l source -r -d "Filter incoming shares by sender ID"
complete -c its-net -n "__fish_seen_subcommand_from client-receive" -l aeh -d "Use Ambient Entropy Harvesting mode"
complete -c its-net -n "__fish_seen_subcommand_from client-receive" -l unwrap -d "Attempt SSS-trapdoor decapsulation"
