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
          overlays = [ rust-overlay.overlays.default ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };

          rustCi = pkgs.rust-bin.stable.latest.default;

          rustDev = rustCi.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
            ];
          };
        in
        {
          devShells = {
            default = pkgs.mkShell {
              packages = [
                rustDev
                # Until https://github.com/rust-lang/cargo/issues/17009 is stable
                pkgs.cargo-cooldown
              ];
            };

            ci = pkgs.mkShell {
              packages = [ rustCi ];
            };
          };
        };
    };
}
