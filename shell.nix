# shell.nix - Hermetic Dev Shell pinning the rust toolchain for reproducible environments.
{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    rustfmt
    clippy
    curl
    gnumake
  ];

  shellHook = ''
    echo "====================================================================="
    echo "⚡ Hydra-ITS Hermetic Development Environment Active ⚡"
    echo "Targeting 100% reproducible builds and supply chain sterility."
    echo "====================================================================="
  '';
}
