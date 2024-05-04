{
  inputs = {
    crane = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "github:ipetkov/crane";
    };
    nixpkgs.url = "nixpkgs/nixos-unstable";
    parts.url = "github:hercules-ci/flake-parts";
    pre-commit-hooks = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "github:cachix/pre-commit-hooks.nix";
    };
  };
  outputs = inputs:
    inputs.parts.lib.mkFlake {inherit inputs;} {
      imports = [inputs.pre-commit-hooks.flakeModule];
      perSystem = {
        config,
        inputs',
        lib,
        pkgs,
        system,
        ...
      }: let
        craneLib = inputs.crane.lib.${system};
      in {
        devShells.default = pkgs.mkShell {
          packages = [pkgs.bacon pkgs.cargo pkgs.clippy];
          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
          shellHook = "${config.pre-commit.installationScript}";
        };
        packages.ghlink = craneLib.buildPackage {
          src = craneLib.cleanCargoSource (craneLib.path ./.);
        };
        pre-commit = {
          settings = {
            hooks.rustfmt.enable = true;
            src = ./.;
          };
        };
      };
      systems = ["aarch64-darwin" "aarch64-linux" "x86_64-darwin" "x86_64-linux"];
    };
}
