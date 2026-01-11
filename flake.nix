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
          libxkbcommon
          wayland
        ];

        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

        package = pkgs.rustPlatform.buildRustPackage {
          pname = cargoToml.package.name;
          version = cargoToml.package.version;
          meta.mainProgram = cargoToml.package.name;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          postFixup = ''
            patchelf --add-rpath ${pkgs.lib.makeLibraryPath buildInputs} $out/bin/${cargoToml.package.name}
          '';
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
