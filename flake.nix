{
  description = "A high-performance Rust crate for processing Gaussian Splatting PLY and SPLAT files";

  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    fenix,
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        f = with fenix.packages.${system};
          combine [
            stable.toolchain
            targets.wasm32-unknown-unknown.stable.rust-std
          ];
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            f
            llvmPackages.bintools
            pkg-config
            cargo
            cargo-fuzz
            wasm-pack
            nodejs_20
            pnpm
            maturin
            uv
          ];
          CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "lld";
        };
      }
    );
}
