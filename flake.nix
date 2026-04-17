{
  description = "Build a cargo workspace";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      advisory-db,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        inherit (pkgs) lib;

        craneLib = (crane.mkLib pkgs).overrideToolchain (
          p:
          p.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" "llvm-tools-preview" ];
          }
        );

        src = lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [
            (lib.fileset.fromSource (craneLib.cleanCargoSource ./.) )
            ./frontend/app/tauri.conf.json
            ./frontend/app/capabilities
            ./frontend/app/build.rs
            ./frontend/app/icons
            ./migrations
          ];
        };

        commonArgs = {
          inherit src;
          strictDeps = true;
          stdenv = p: p.clangStdenv;

          buildInputs = [
            pkgs.z3
            pkgs.pkg-config
            pkgs.glib
            pkgs.gtk3
            pkgs.librsvg
            pkgs.webkitgtk_4_1
          ]
          ++ lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];

          nativeBuildInputs = [
            pkgs.mold
            pkgs.pkg-config
            pkgs.cmake
            pkgs.python314
          ];

          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

          # Additional environment variables can be set directly
          # MY_CUSTOM_VAR = "some value";
        };
      in
      {
        devShells.default = craneLib.devShell {
          inherit (commonArgs) buildInputs nativeBuildInputs;
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

          shellHook = ''
            export XDG_DATA_DIRS="$GSETTINGS_SCHEMAS_PATH"
            export LD_LIBRARY_PATH="${pkgs.webkitgtk_4_1}/lib:$LD_LIBRARY_PATH"
          '';

          packages = with pkgs; [
            cargo-hakari
            cargo-llvm-cov
            cargo-tauri
            tombi
            nodejs
            pnpm
            diesel-cli
            clang
            mold
            pkg-config
            wrapGAppsHook4
            python3
            dbus
            glib
            gtk3
            librsvg
            webkitgtk_4_1
          ];
        };
      }
    );
}
