{
  inputs = {
    nix-go = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "github:matthewdargan/nix-go";
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
        ...
      }: {
        devShells.default = pkgs.mkShell {
          packages = [inputs'.nix-go.packages.go];
          shellHook = "${config.pre-commit.installationScript}";
        };
        packages = {
          ghlink = inputs'.nix-go.legacyPackages.buildGoModule {
            meta = with lib; {
              description = "Create GitHub permanent links to specified file lines";
              homepage = "https://github.com/matthewdargan/ghlink";
              license = licenses.bsd3;
              maintainers = with maintainers; [matthewdargan];
            };
            pname = "ghlink";
            src = ./.;
            vendorHash = null;
            version = "0.2.3";
          };
        };
        pre-commit = {
          settings = {
            hooks = {
              golangci-lint = {
                enable = true;
                package = inputs'.nix-go.packages.golangci-lint;
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
