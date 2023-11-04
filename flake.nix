{
  inputs.flake-parts.url = "github:hercules-ci/flake-parts";
  inputs.treefmt-nix.url = "github:numtide/treefmt-nix";
  inputs.pre-commit-hooks-nix.url = "github:cachix/pre-commit-hooks.nix";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.fenix = {
    url = "github:nix-community/fenix";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs @ {
    self,
    flake-parts,
    treefmt-nix,
    pre-commit-hooks-nix,
    fenix,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs self;} {
      imports = [
        treefmt-nix.flakeModule
        pre-commit-hooks-nix.flakeModule
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
        config,
        system,
        ...
      }: let
        rustToolchain = with fenix.packages.${system};
          combine [
            (complete.withComponents [
              "cargo"
              "clippy"
              "rust-src"
              "rustc"
              "rustfmt"
            ])
            rust-analyzer
          ];

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };

        rename = pkgs.writeShellApplication {
          name = "rename";
          runtimeInputs = [
            pkgs.fd
            pkgs.coreutils
            pkgs.nixFlakes
            pkgs.jq
          ];
          text = ''
            newName="$1"
            pushd "$(nix flake metadata --json 2>/dev/null | jq .original.path -r)"
            # This is only used to prevent the literal name from appearing here, as the rename would find it!
            oldName=$(echo 'change_my_name' | sed 's/_/-/g')
            stat "nix/packages/$oldName.nix" 2>/dev/null >/dev/null || echo -e "\033[31mCan only be done once\033[0m"
            stat "nix/packages/$oldName.nix" 2>/dev/null >/dev/null || exit 1
            fd --type f --exec sed "s/aba2sat/$newName/g" -i '{}'
            mv nix/packages/aba2sat.nix "nix/packages/$newName.nix"
          '';
        };
      in {
        pre-commit.check.enable = true;
        pre-commit.settings.hooks.markdownlint.enable = true;
        pre-commit.settings.hooks.nil.enable = true;
        pre-commit.settings.hooks.format = {
          enable = true;
          entry = "${self'.formatter}/bin/fmt";
          pass_filenames = false;
        };
        pre-commit.settings.hooks.my-clippy = {
          enable = true;
          name = "clippy";
          description = "Lint Rust code.";
          entry = "${rustToolchain}/bin/cargo-clippy clippy --offline -- -D warnings";
          files = "\\.rs$";
          pass_filenames = false;
        };
        pre-commit.settings.hooks.my-cargo-check = {
          enable = true;
          description = "check the cargo package for errors.";
          entry = "${rustToolchain}/bin/cargo check --offline";
          files = "\\.rs$";
          pass_filenames = false;
        };

        treefmt.projectRootFile = "flake.nix";
        treefmt.programs = {
          rustfmt.enable = true;
          alejandra.enable = true;
        };
        treefmt.flakeFormatter = true;

        packages.aba2sat = pkgs.callPackage ./nix/packages/aba2sat.nix {inherit rustPlatform;};
        packages.default = self'.packages.aba2sat;

        devShells.default = pkgs.mkShell {
          name = "aba2sat";
          shellHook = ''
            ${config.pre-commit.installationScript}
            # This is only used to prevent the literal name from appearing here, as the rename would find it!
            oldName=$(echo 'change_my_name' | sed 's/_/-/g')
            echo -e 1>&2 "\n\n  Welcome to the development shell!"
            stat "nix/packages/$oldName.nix" 2>/dev/null >/dev/null && echo -e 1>&2 "\n  \033[31mChange this projects name with \033[1mrename NEW-NAME\033[0m"
            echo -e 1>&2 "\n"
          '';
          nativeBuildInputs = [
            config.treefmt.package
            pkgs.cargo-workspaces
            pkgs.nil
            rustToolchain
            rename
          ];
          RUST_LOG = "trace";
        };
        devShells.pre-commit = config.pre-commit.devShell;
      };
    };
}
