{
  description = "libZetta Development Environment";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    devshell = {
      url = "github:numtide/devshell/master";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
  };
  outputs =
    { self
    , nixpkgs
    , fenix
    , flake-utils
    , devshell
    , pre-commit-hooks
    , ...
    }:
    flake-utils.lib.eachSystem [
      "x86_64-linux"
      "aarch64-linux"
      "aarch64-darwin"
    ]
      (system:
      let
        overlays = [ devshell.overlay fenix.overlay ];
        pkgs = import nixpkgs { inherit system overlays; };
      in
      with pkgs; {
        checks = {
          pre-commit-check = pre-commit-hooks.lib.${system}.run {
            src = ./.;
            hooks = {
              nixpkgs-fmt.enable = true;
              shellcheck.enable = true;
              statix.enable = true;
              nix-linter.enable = true;
            };
          };
        };

        devShell = clangStdenv.mkDerivation rec {
          inherit (self.checks.${system}.pre-commit-check) shellHook;
          name = "libzetta-env";
          nativeBuildInputs = [
            (pkgs.fenix.complete.withComponents [
              "cargo"
              "clippy"
              "rust-src"
              "rustc"
              "rustfmt"
            ])
            rust-analyzer-nightly
            bacon
            cargo-cache
            cargo-deny
            cargo-diet
            cargo-sort
            cargo-sweep
            cargo-wipe
            cargo-outdated
            cargo-release
            git-cliff
            cmake
            gnumake
            openssl.dev
            pkg-config
            nixpkgs-fmt
            zfs.dev
            just
            vagrant
          ];
          PROJECT_ROOT = builtins.toString ./.;
        };
      });
}

