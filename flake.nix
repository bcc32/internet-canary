{
  description = "internet-canary";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    rust-overlay.url = "github:oxalica/rust-overlay";
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
          ] ++ packages.default.buildInputs;
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };

        devShells.udeps = mkShell {
          buildInputs = [
            cargo-udeps
            pkgs.rust-bin.nightly.latest.default
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };

        packages.default = naersk-lib.buildPackage {
          src = ./.;
          buildInputs =
            lib.optional stdenv.isDarwin darwin.apple_sdk.frameworks.Security
            ++ lib.optionals stdenv.isLinux [
              openssl
              pkg-config
            ];
        };
      }
    );
}
