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

          rustDev = pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
            ];
          };

          # Surprisingly not in nixpkgs
          cargo-cooldown =
            let
              pname = "cargo-cooldown";
              version = "0.3.3";
            in
            pkgs.rustPlatform.buildRustPackage {
              inherit pname version;
              src = pkgs.fetchFromGitHub {
                owner = "dertin";
                repo = pname;
                rev = "v${version}";
                hash = "sha256-rAIZVKF65hsT3NRzaYAfXacp5uZ2NfvVe56zdbOMKSk=";
              };

              cargoHash = "sha256-iu/VCs7uJtaMKt80ZgnuB6fq/C0nh7ArTeEajaw2xR8=";

              # Some lockfile tests write to .cache
              # Fix from https://discourse.nixos.org/t/temporary-writable-directory-for-builds/34907/10
              nativeCheckInputs = [
                pkgs.writableTmpDirAsHomeHook
              ];

              # Some lockfile tests require certificates
              # Fix inspired from https://github.com/NixOS/nixpkgs/blob/b5aa0fbd538984f6e3d201be0005b4463d8b09f8/pkgs/development/python-modules/pygit2/default.nix#L51-L53
              env.SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
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
              packages = [
                rustDev
                # Until https://github.com/rust-lang/cargo/issues/17009 is stable
                cargo-cooldown
              ];
            };

            ci = mkShellNoHardening {
              packages = [ rustCi ];
            };
          };
        };
    };
}
