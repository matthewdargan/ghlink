{
  inputs = {
    nix-go.url = "github:matthewdargan/nix-go";
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
        pkgs,
        ...
      }: {
        devShells.default = pkgs.mkShell {
          packages = [inputs.nix-go.packages.${pkgs.system}.go];
          shellHook = "${config.pre-commit.installationScript}";
        };
        packages = {
          ghlink = pkgs.buildGoModule {
            pname = "ghlink";
            src = ./.;
            vendorHash = null;
            version = "0.2.0";
          };
        };
        pre-commit = {
          settings = {
            hooks = {
              golangci-lint = {
                enable = true;
                package = inputs.nix-go.packages.${pkgs.system}.golangci-lint;
              };
              gotest.enable = true;
            };
            src = ./.;
          };
        };
      };
      systems = ["aarch64-darwin" "aarch64-linux" "x86_64-darwin" "x86_64-linux"];
    };
}
