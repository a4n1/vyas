{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix.url = "github:nix-community/fenix";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, fenix, utils }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        fenixPkgs = fenix.packages.${system};
        f = with fenixPkgs; combine [
          stable.toolchain
          rust-analyzer
        ];
      in
      {
        devShell = with pkgs; mkShell {
          buildInputs = [
            f
          ];
          shellHook = ''
            export RUST_SRC_PATH="${fenixPkgs.stable.rust-src}/lib/rustlib/src/rust/src"
          '';
        };
      }
    );
}

