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

        src = craneLib.cleanCargoSource ./.;

        # Common arguments can be set here to avoid repeating them later
        commonArgs = {
          inherit src;
          strictDeps = true;
          stdenv = p: p.clangStdenv;

          buildInputs = [
            # Add additional build inputs here
            pkgs.z3
          ]
          ++ lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
          ];

          nativeBuildInputs = [
            pkgs.mold
            pkgs.clang
            pkgs.cargo-llvm-cov
          ];

          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

          # Additional environment variables can be set directly
          # MY_CUSTOM_VAR = "some value";
        };

        # Build *just* the cargo dependencies (of the entire workspace),
        # so we can reuse all of that work (e.g. via cachix) when running in CI
        # It is *highly* recommended to use something like cargo-hakari to avoid
        # cache misses when building individual top-level-crates
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        individualCrateArgs = commonArgs // {
          inherit cargoArtifacts;
          inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
          # NB: we disable tests since we'll run them all via cargo-nextest
          doCheck = false;
        };

        fileSetForCrate =
          crate:
          lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              ./migrations
              (craneLib.fileset.commonCargoSources ./crates/model)
              (craneLib.fileset.commonCargoSources ./crates/hack)
              (craneLib.fileset.commonCargoSources ./crates/solver)
              (craneLib.fileset.commonCargoSources ./crates/backend)
              (craneLib.fileset.commonCargoSources crate)
            ];
          };

        # Build the top-level crates of the workspace as individual derivations.
        # This allows consumers to only depend on (and build) only what they need.
        # Though it is possible to build the entire workspace as a single derivation,
        # so this is left up to you on how to organize things
        #
        # Note that the cargo workspace must define `workspace.members` using wildcards,
        # otherwise, omitting a crate (like we do below) will result in errors since
        # cargo won't be able to find the sources for all members.
        # my-cli = craneLib.buildPackage (
        #   individualCrateArgs
        #   // {
        #     pname = "my-cli";
        #     cargoExtraArgs = "-p my-cli";
        #     src = fileSetForCrate ./crates/my-cli;
        #   }
        # );
        # my-server = craneLib.buildPackage (
        #   individualCrateArgs
        #   // {
        #     pname = "my-server";
        #     cargoExtraArgs = "-p my-server";
        #     src = fileSetForCrate ./crates/my-server;
        #   }
        # );
      in
      {
        checks = {
          # Build the crates as part of `nix flake check` for convenience
          # inherit my-server;

          # Run clippy (and deny all warnings) on the workspace source,
          # again, reusing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          exam-timetable-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          exam-timetable-doc = craneLib.cargoDoc (
            commonArgs
            // {
              inherit cargoArtifacts;
              # This can be commented out or tweaked as necessary, e.g. set to
              # `--deny rustdoc::broken-intra-doc-links` to only enforce that lint
              env.RUSTDOCFLAGS = "--deny warnings";
            }
          );

          # Check formatting
          exam-timetable-fmt = craneLib.cargoFmt {
            inherit src;
          };

          exam-timetable-toml-fmt = craneLib.taploFmt {
            src = pkgs.lib.sources.sourceFilesBySuffices src [ ".toml" ];
            # taplo arguments can be further customized below as needed
            # taploExtraArgs = "--config ./taplo.toml";
          };

          # Audit dependencies
          exam-timetable-audit = craneLib.cargoAudit {
            inherit src advisory-db;
          };

          # Audit licenses
          exam-timetable-deny = craneLib.cargoDeny {
            inherit src;
          };

          # Run tests with cargo-nextest
          # Consider setting `doCheck = false` on other crate derivations
          # if you do not want the tests to run twice
          exam-timetable-nextest = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
              cargoNextestPartitionsExtraArgs = "--no-tests=pass";
            }
          );

          # Generate LLVM coverage summary for the workspace
          exam-timetable-coverage = craneLib.mkCargoDerivation {
            inherit src;
            pname = "exam-timetable-coverage";
            cargoArtifacts = null;
            doInstallCargoArtifacts = false;

            nativeBuildInputs = [
              pkgs.cargo-llvm-cov
              pkgs.rust-bin.stable.latest.default
              pkgs.clang
              pkgs.mold
            ];

            buildPhaseCargoCommand = ''
              mkdir -p $out
              cargo llvm-cov nextest --workspace --all-features --summary-only > $out/summary.txt
              cargo llvm-cov nextest --workspace --all-features --lcov --output-path $out/lcov.info
              cargo llvm-cov nextest --workspace --all-features --cobertura --output-path $out/cobertura.xml
            '';
          };

          # Generate browsable HTML coverage report
          exam-timetable-coverage-html = craneLib.mkCargoDerivation {
            inherit src;
            pname = "exam-timetable-coverage-html";
            cargoArtifacts = null;
            doInstallCargoArtifacts = false;

            nativeBuildInputs = [
              pkgs.cargo-llvm-cov
              pkgs.rust-bin.stable.latest.default
              pkgs.clang
              pkgs.mold
            ];

            buildPhaseCargoCommand = ''
              cargo llvm-cov nextest --workspace --all-features --html --output-dir html
              mkdir -p $out
              cp -r html/* $out/
            '';
          };

          # Ensure that cargo-hakari is up to date
          exam-timetable-hakari = craneLib.mkCargoDerivation {
            inherit src;
            pname = "exam-timetable-hakari";
            cargoArtifacts = null;
            doInstallCargoArtifacts = false;

            buildPhaseCargoCommand = ''
              cargo hakari generate --diff  # workspace-hack Cargo.toml is up-to-date
              cargo hakari manage-deps --dry-run  # all workspace crates depend on workspace-hack
              cargo hakari verify
            '';

            nativeBuildInputs = [
              pkgs.cargo-hakari
            ];
          };
        };

        packages = {
          # inherit my-server;
          coverage = self.checks.${system}.exam-timetable-coverage;
          coverage-html = self.checks.${system}.exam-timetable-coverage-html;
        };

        apps = {
          # my-server = flake-utils.lib.mkApp {
          #   drv = my-server;
          # };
          coverage = flake-utils.lib.mkApp {
            drv = self.packages.${system}.coverage;
          };
          coverage-html = flake-utils.lib.mkApp {
            drv = self.packages.${system}.coverage-html;
          };
        };

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};

          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

          packages = [
            pkgs.cargo-hakari
            pkgs.cargo-llvm-cov
            pkgs.sqlx-cli
            pkgs.nixd
            pkgs.clang
            pkgs.mold
            # for rustrover
            pkgs.rustup
          ];
        };
      }
    );
}
