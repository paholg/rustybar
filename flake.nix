{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      crane,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [
          (import rust-overlay)
        ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rust = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-analyzer"
            "rust-src"
          ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.rust-bin.stable.latest.default);

        buildInputs = with pkgs; [
          wayland
          libxkbcommon
          vulkan-loader
        ];

        crate = craneLib.buildPackage {
          inherit buildInputs;
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;
        };
      in
      {
        checks = {
          inherit crate;
        };
        packages = {
          default = crate;
          inherit crate;
        };
        devShells.default = pkgs.mkShell {
          inherit buildInputs;
          packages = [
            pkgs.just
            rust
          ];

          env = {
            RUST_BACKTRACE = 1;
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
          };
        };
      }
    );
}
