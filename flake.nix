{
  description = "internet-canary";

  inputs = {
    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      naersk,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust-bin = pkgs.rust-bin.stable.latest;
        naersk-lib = pkgs.callPackage naersk {
          cargo = rust-bin.minimal;
          rustc = rust-bin.minimal;
        };
      in
      with pkgs;
      rec {
        devShells.default = mkShell {
          buildInputs = [
            cargo-outdated
            rust-analyzer
            rust-bin.default
          ]
          ++ packages.default.buildInputs;
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };

        devShells.udeps = mkShell {
          buildInputs = [
            cargo-udeps
            cargo-edit
            pkgs.rust-bin.nightly.latest.default
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };

        packages.default = naersk-lib.buildPackage {
          src = ./.;
          buildInputs =
            lib.optional stdenv.hostPlatform.isDarwin darwin.apple_sdk.frameworks.Security
            ++ lib.optionals stdenv.hostPlatform.isLinux [
              openssl
              pkg-config
            ];
        };
      }
    );
}
