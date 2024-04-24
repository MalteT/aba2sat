{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    flake-parts.url = "github:hercules-ci/flake-parts";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };

    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    flake-parts,
    crane,
    fenix,
    flake-utils,
    advisory-db,
    rust-overlay,
  }:
    flake-parts.lib.mkFlake {inherit inputs self;} {
      imports = [
      ];

      systems = [
        "x86_64-linux"
      ];

      flake.hydraJobs.packages.x86_64-linux = self.packages.x86_64-linux;
      flake.hydraJobs.devShells.x86_64-linux = self.devShells.x86_64-linux;
      flake.hydraJobs.checks.x86_64-linux = self.checks.x86_64-linux;

      perSystem = {
        self',
        pkgs,
        lib,
        config,
        system,
        ...
      }: let
        # Load toolchain from file and extend with rust-src and rust-analyzer + clippy
        rustToolchain = (pkgs.rust-bin.fromRustupToolchainFile (self + /rust-toolchain.toml)).override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
          ];
        };

        # NB: we don't need to overlay our custom toolchain for the *entire*
        # pkgs (which would require rebuidling anything else which uses rust).
        # Instead, we just want to update the scope that crane will use by appending
        # our specific toolchain there.
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        src = craneLib.cleanCargoSource (craneLib.path ./.);

        # Common arguments can be set here to avoid repeating them later
        commonArgs = {
          inherit src;
          strictDeps = true;

          buildInputs =
            [
              # Add additional build inputs here
            ]
            ++ lib.optionals pkgs.stdenv.isDarwin [
              # Additional darwin specific inputs can be set here
              pkgs.libiconv
            ];

          # Additional environment variables can be set directly
          # MY_CUSTOM_VAR = "some value";
        };

        craneLibLLvmTools =
          craneLib.overrideToolchain
          (fenix.packages.${system}.complete.withComponents [
            "cargo"
            "llvm-tools"
            "rustc"
          ]);

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        aba2sat = craneLib.buildPackage (commonArgs
          // {
            inherit cargoArtifacts;
          });
      in {
        _module.args.pkgs = import nixpkgs {
          inherit system;
          overlays = [
            (import rust-overlay)
          ];
        };

        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit aba2sat;

          # Run clippy (and deny all warnings) on the crate source,
          # again, reusing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          aba2sat-clippy = craneLib.cargoClippy (commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            });

          aba2sat-doc = craneLib.cargoDoc (commonArgs
            // {
              inherit cargoArtifacts;
            });

          # Check formatting
          aba2sat-fmt = craneLib.cargoFmt {
            inherit src;
          };

          # Audit dependencies
          aba2sat-audit = craneLib.cargoAudit {
            inherit src advisory-db;
          };

          # Audit licenses
          aba2sat-deny = crane.lib.${system}.cargoDeny {
            inherit src;
          };

          # Run tests with cargo-nextest
          # Consider setting `doCheck = false` on `aba2sat` if you do not want
          # the tests to run twice
          aba2sat-nextest = craneLib.cargoNextest (commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
            });
          };

          packages = {
            default = aba2sat;
            aspforaba = pkgs.callPackage ./nix/packages/aspforaba.nix { inherit (self'.packages) clingo; };
            clingo = pkgs.callPackage ./nix/packages/clingo.nix { };
          } // lib.optionalAttrs (!pkgs.stdenv.isDarwin) {
            aba2sat-llvm-coverage = craneLibLLvmTools.cargoLlvmCov (commonArgs // {
              inherit cargoArtifacts;
            });
          };

          apps.default = flake-utils.lib.mkApp {
            drv = aba2sat;
          };

          devShells.default = craneLib.devShell {
            # Inherit inputs from checks.
            checks = self.checks.${system};

            RUST_LOG = "trace";

            inputsFrom = [ ];

            packages = [
              pkgs.hyperfine
              pkgs.lldb
              pkgs.nil
              pkgs.nodejs
              pkgs.pre-commit
              pkgs.shellcheck
              pkgs.shfmt
              self'.packages.aspforaba
            ];
          };
        };
      };
}
