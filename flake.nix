{
  inputs = {
    crane = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "github:ipetkov/crane";
    };
    fenix = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "github:nix-community/fenix";
    };
    nix-filter.url = "github:numtide/nix-filter";
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
        craneLib = inputs.crane.lib.${system}.overrideToolchain rustToolchain;
        rustToolchain = inputs'.fenix.packages.stable.toolchain;
      in {
        devShells.default = pkgs.mkShell {
          packages = [pkgs.bacon rustToolchain];
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/src";
          shellHook = "${config.pre-commit.installationScript}";
        };
        packages.ghlink = craneLib.buildPackage {
          nativeBuildInputs = lib.optionals pkgs.stdenv.isDarwin [pkgs.libiconv];
          src = inputs.nix-filter.lib {
            include = [
              "Cargo.lock"
              "Cargo.toml"
              "src"
            ];
            root = ./.;
          };
        };
        pre-commit = {
          settings = {
            hooks = {
              alejandra.enable = true;
              deadnix.enable = true;
              rustfmt.enable = true;
              statix.enable = true;
            };
            src = ./.;
          };
        };
      };
      systems = ["aarch64-darwin" "aarch64-linux" "x86_64-darwin" "x86_64-linux"];
    };
}
