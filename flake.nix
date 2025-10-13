{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-25.05";
    flake-parts.url = "github:hercules-ci/flake-parts";
    naersk.url = "github:nix-community/naersk";
    # Just used for rustChannelOf
    nixpkgs-mozilla.url = "github:mozilla/nixpkgs-mozilla";
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      flake-parts,
      naersk,
      nixpkgs-mozilla,
    }:
    flake-parts.lib.mkFlake { inherit inputs; } (
      { ... }:
      {
        systems = [
          "x86_64-linux"
          "aarch64-linux"
        ];

        perSystem =
          {
            system,
            config,
            pkgs,
            ...
          }:
          let
            toolchain =
              (pkgs.rustChannelOf {
                rustToolchain = ./rust-toolchain.toml;
                sha256 = "sha256-SJwZ8g0zF2WrKDVmHrVG3pD2RGoQeo24MEXnNx5FyuI=";
              }).rust;

            naersk' = pkgs.callPackage naersk {
              cargo = toolchain;
              rustc = toolchain;
            };
          in
          {
            _module.args.pkgs = import nixpkgs {
              inherit system;
              overlays = [
                (import nixpkgs-mozilla)
              ];
              config = { };
            };

            packages.default = naersk'.buildPackage {
              src = ./.;
            };

            devShells.default = pkgs.mkShell {
              nativeBuildInputs = with pkgs; [
                cargo
                pkg-config
                toolchain
              ];
              RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
            };
          };
      }
    );
}
