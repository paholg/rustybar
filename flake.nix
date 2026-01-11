{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustDev = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-analyzer"
            "rust-src"
          ];
        };

        buildInputs = with pkgs; [
          wayland
          libxkbcommon
          vulkan-loader
        ];

        package = pkgs.rustPlatform.buildRustPackage {
          pname = "rustybar";
          version = "0.2.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };
      in
      {
        packages.default = package;
        devShells.default = pkgs.mkShell {
          inherit buildInputs;
          packages = [
            pkgs.just
            rustDev
          ];

          env = {
            RUST_BACKTRACE = 1;
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
          };
        };
      }
    );
}
