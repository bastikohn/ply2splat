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
        
        toolchain = fenix.packages.${system}.stable.toolchain;
        
        rustPlatform = pkgs.makeRustPlatform {
          cargo = toolchain;
          rustc = toolchain;
        };

        manifest = (pkgs.lib.importTOML ./Cargo.toml).workspace.package;

        ply2splat = rustPlatform.buildRustPackage {
          pname = "ply2splat";
          version = manifest.version;
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          
          # Only build the CLI binary
          cargoBuildFlags = ["-p" "ply2splat"];
          
          # Disable tests in the flake build for speed/reliability
          doCheck = false;
        };

        f = with fenix.packages.${system};
          combine [
            stable.toolchain
            targets.wasm32-unknown-unknown.stable.rust-std
            targets.wasm32-wasip1-threads.stable.rust-std
          ];
      in {
        packages.default = ply2splat;
        packages.ply2splat = ply2splat;
        
        apps.default = flake-utils.lib.mkApp {
          drv = ply2splat;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            f
            llvmPackages.bintools
            pkg-config
            cargo
            cargo-fuzz
            wasm-pack
            nodejs_22
            pnpm
            maturin
            uv
          ];
          CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "lld";
        };
      }
    );
}