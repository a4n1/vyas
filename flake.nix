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
        rustToolchain = with fenixPkgs; combine [
          latest.toolchain
          targets.wasm32-unknown-unknown.latest.rust-std
          rust-analyzer
        ];
      in
      {
        devShell = with pkgs; mkShell {
          buildInputs = [
            rustToolchain
            wasm-pack
            openssl
            nodejs_24
            corepack
            typescript-language-server
          ];
          shellHook = ''
            export RUST_SRC_PATH="${fenixPkgs.latest.rust-src}/lib/rustlib/src/rust/src"
          '';
        };
      }
    );
}

