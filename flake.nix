{
  inputs.flake-parts.url = "github:hercules-ci/flake-parts";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.fenix = {
    url = "github:nix-community/fenix";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    inputs @ { self
    , flake-parts
    , fenix
    , ...
    }:
    flake-parts.lib.mkFlake { inherit inputs self; } {
      imports = [
      ];

      systems = [
        "x86_64-linux"
      ];

      flake.hydraJobs.packages.x86_64-linux = self.packages.x86_64-linux;
      flake.hydraJobs.devShells.x86_64-linux = self.devShells.x86_64-linux;
      flake.hydraJobs.checks.x86_64-linux = self.checks.x86_64-linux;

      perSystem =
        { self'
        , pkgs
        , config
        , system
        , ...
        }:
        let
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
        in
        {
          packages.aba2sat = pkgs.callPackage ./nix/packages/aba2sat.nix { inherit rustPlatform; };
          packages.aspforaba = pkgs.callPackage ./nix/packages/aspforaba.nix { };
          packages.clingo = pkgs.callPackage ./nix/packages/clingo.nix { };
          packages.default = self'.packages.aba2sat;

          devShells.default = pkgs.mkShell {
            name = "aba2sat";
            nativeBuildInputs = [
              pkgs.cargo-workspaces
              pkgs.nil
              pkgs.pre-commit
              pkgs.nodejs
              pkgs.shellcheck
              pkgs.shfmt
              rustToolchain
              rename
              self'.packages.aspforaba
            ];
            RUST_LOG = "trace";
          };
          devShells.pre-commit = config.pre-commit.devShell;
        };
    };
}
