{
  description = "Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs =
    inputs@{
      nixpkgs,
      rust-overlay,
      flake-parts,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];

      perSystem =
        { system, ... }:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };

          rustMinimal = pkgs.rust-bin.stable.latest.minimal;

          rustDev = pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
            ];
          };

          # Disable unwanted hardening flags that can cause unexpected behavor
          # https://github.com/NixOS/nixpkgs/issues/18995#issuecomment-4061380978
          mkShellNoHardening =
            args:
            pkgs.mkShell (
              {
                hardeningDisable = [ "all" ];
                NIX_ENFORCE_NO_NATIVE = false;
              }
              // args
            );
        in
        {
          devShells = {
            default = mkShellNoHardening {
              nativeBuildInputs = [ rustDev ];
            };

            ci = mkShellNoHardening {
              nativeBuildInputs = [ rustMinimal ];
            };
          };
        };
    };
}
