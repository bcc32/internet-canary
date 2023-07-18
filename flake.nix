{
  description = "internet-canary";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, naersk, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs { inherit system overlays; };
        naersk-lib = pkgs.callPackage naersk { };
      in with pkgs; {
        devShell = mkShell {
          buildInputs = [ ed rust-analyzer rust-bin.beta.latest.default ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };

        defaultPackage = naersk-lib.buildPackage { src = ./.; };
      });
}
