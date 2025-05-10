{
  description = "xepher-fetch";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    treefmt-nix.url = "github:numtide/treefmt-nix";
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      ...
    }:
    let
      overlay = import ./nix/overlay.nix;
    in
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      # Add supported platform here
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];

      flake = {
        overlays = {
          default = overlay;
        };
      };

      imports = [
        # Add treefmt flake module to automatically configure and add formatter to this flake
        inputs.treefmt-nix.flakeModule
      ];

      perSystem =
        { system, ... }:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [
              overlay
            ];
          };
        in
        {
          # Override the default "pkgs" attribute in per-system config.
          _module.args.pkgs = pkgs;

          # Although the pkgs attribute is already override, but I am afraid
          # that the magical evaluation of "pkgs" is confusing, and will lead
          # to debug hell. So here we use the "pkgs" in "let-in binding" to
          # explicitly told every user we are using an overlayed version of
          # nixpkgs.
          legacyPackages = pkgs;

          devShells.default = pkgs.mkShell {
            buildInputs = [
              pkgs.cargo
              pkgs.rust-analyzer
            ];
          };

          treefmt = {
            projectRootFile = "flake.nix";
            settings.on-unmatched = "debug";
            programs = {
              nixfmt.enable = true;
            };
          };
        };
    };
}
