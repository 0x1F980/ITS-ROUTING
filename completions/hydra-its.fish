# Fish completion for hydra-its CLI

# Disable file completion by default for commands
complete -c hydra-its -f

# Global flags
complete -c hydra-its -s c -l config -r -g -d "Specify path to config.toml"
complete -c hydra-its -s h -l help -d "Show help menu"
complete -c hydra-its -s v -l version -d "Show tool version"

# Commands
complete -c hydra-its -n "__fish_use_subcommand" -a start-node -d "Start an active VPS routing node"
complete -c hydra-its -n "__fish_use_subcommand" -a client-send -d "Send an Onion-encrypted packet"
complete -c hydra-its -n "__fish_use_subcommand" -a client-receive -d "Receive and reconstruct packets"
complete -c hydra-its -n "__fish_use_subcommand" -a status-audit -d "Run diagnostic telemetry"
complete -c hydra-its -n "__fish_use_subcommand" -a verify-path -d "Verify homomorphic routing path"
complete -c hydra-its -n "__fish_use_subcommand" -a list-peers -d "Show current finite field routing table"

# Subcommand-specific arguments
# start-node
complete -c hydra-its -n "__fish_seen_subcommand_from start-node" -l chaff-rate -r -d "Tick rate for dummy traffic in ms"
complete -c hydra-its -n "__fish_seen_subcommand_from start-node" -l port -r -d "Local port for bare-metal UDP binder"
complete -c hydra-its -n "__fish_seen_subcommand_from start-node" -l daemonize -d "Run routing node in the background"

# client-send
complete -c hydra-its -n "__fish_seen_subcommand_from client-send" -l msg -r -d "The secret message to fragment and encrypt"
complete -c hydra-its -n "__fish_seen_subcommand_from client-send" -l dest -r -d "The recipient ID in Z_{2^31-1}"
complete -c hydra-its -n "__fish_seen_subcommand_from client-send" -l aeh -d "Enable Ambient Entropy Harvesting mode"

# client-receive
complete -c hydra-its -n "__fish_seen_subcommand_from client-receive" -l source -r -d "Filter incoming shares by sender ID"
complete -c hydra-its -n "__fish_seen_subcommand_from client-receive" -l aeh -d "Use Ambient Entropy Harvesting mode"
complete -c hydra-its -n "__fish_seen_subcommand_from client-receive" -l unwrap -d "Attempt SSS-trapdoor decapsulation"
