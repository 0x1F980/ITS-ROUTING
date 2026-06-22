{
  description = "ITS-routing — UES Monocell Pool transport (constitution build)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc cargo pkg-config openssl
          ];
          shellHook = ''
            echo "ITS-routing nix devShell — run: cargo build -p its_routing --release"
          '';
        };

        packages.its-routing = pkgs.rustPlatform.buildRustPackage {
          pname = "its-routing";
          version = "2.0.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [ openssl ];
          doCheck = false;
        };

        packages.default = self.packages.${system}.its-routing;
      });
}
