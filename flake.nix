{
  inputs.flake-parts.url = "github:hercules-ci/flake-parts";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.fenix = {
    url = "github:nix-community/fenix";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs @ {
    self,
    flake-parts,
    fenix,
    ...
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
      in {
        packages.aba2sat = pkgs.callPackage ./nix/packages/aba2sat.nix {inherit rustPlatform;};
        packages.aspforaba = pkgs.callPackage ./nix/packages/aspforaba.nix {};
        packages.clingo = pkgs.callPackage ./nix/packages/clingo.nix {};
        packages.default = self'.packages.aba2sat;

        devShells.default = pkgs.mkShell {
          name = "aba2sat";
          nativeBuildInputs = [
            pkgs.alejandra
            pkgs.cargo-workspaces
            pkgs.nil
            pkgs.nodejs
            pkgs.pre-commit
            pkgs.shellcheck
            pkgs.shfmt
            rustToolchain
            self'.packages.aspforaba
          ];
          RUST_LOG = "trace";
        };
      };
    };
}
