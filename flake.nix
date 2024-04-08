{
  description = "Game Theory labs";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = { nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        rust = (pkgs.rustChannelOf { channel = "stable"; }).default.override {
          extensions = [ "rust-analyzer" "rust-src" "clippy" ];
        };
      in { devShell = pkgs.mkShell { nativeBuildInputs = [ rust ]; }; });
}
