let
  pkgs = import <nixpkgs> {};

  # Overlay for nightly rust
  rust-overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  nixpkgs = import <nixpkgs> { overlays = [ rust-overlay ]; };

  project_name = "uvmapper";
  log_level = "info";
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    nixpkgs.rust-bin.nightly.latest.default
    nixpkgs.rust-analyzer

    # stable rust setup
    #rustc cargo rustfmt rust-analyzer cargo-outdated clippy

    # Utils
    just linuxPackages.perf hotspot

    # Map editor
    tiled
  ];

  RUST_BACKTRACE = "1";
  RUST_LOG = "${project_name}=${log_level}";
}
