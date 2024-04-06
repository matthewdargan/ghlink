{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    parts.url = "github:hercules-ci/flake-parts";
  };
  outputs = inputs:
    inputs.parts.lib.mkFlake {inherit inputs;} {
      perSystem = {pkgs, ...}: {
        packages = {
          ghlink = pkgs.buildGoModule {
            pname = "ghlink";
            src = ./.;
            vendorHash = null;
            version = "0.1.1";
          };
        };
      };
      systems = ["aarch64-darwin" "aarch64-linux" "x86_64-darwin" "x86_64-linux"];
    };
}
