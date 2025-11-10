{
  description = "KOReader Sync Server";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-parts.url = "github:hercules-ci/flake-parts";

    git-hooks-nix = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];

      imports = [
        inputs.git-hooks-nix.flakeModule
        inputs.treefmt-nix.flakeModule
      ];

      perSystem = let
        targets = [
          "aarch64-unknown-linux-gnu"
          "aarch64-unknown-linux-musl"

          "armv7-unknown-linux-gnueabihf"
          "armv7-unknown-linux-musleabihf"

          "x86_64-unknown-linux-gnu"
          "x86_64-unknown-linux-musl"

          "aarch64-apple-darwin"
          "x86_64-apple-darwin"

          "x86_64-pc-windows-gnu"
        ];
      in
        {
          config,
          system,
          ...
        }: let
          overlays = [(import inputs.rust-overlay)];
          pkgs = import inputs.nixpkgs {
            inherit system overlays;
          };

          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = ["rust-src" "rust-analyzer"];
            inherit targets;
          };

          buildInputs = with pkgs;
            [
              openssl
              pkg-config
            ]
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.Security
              pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            ];
        in {
          devShells.default = pkgs.mkShell {
            inherit buildInputs;
            nativeBuildInputs = with pkgs; [
              rustToolchain
              bacon
              cargo-edit
              cargo-zigbuild
              cargo-tarpaulin
              cargo-audit
              cargo-deny

              zig

              clang
              lld
            ];

            # CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS = "-C linker=clang -C link-arg=-fuse-ld=lld";
            CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = "zig-cc";

            shellHook = ''
              ${config.pre-commit.installationScript}

              echo "Rust development environment loaded"
              echo "Rust version: $(rustc --version)"
              echo ""
              echo "Cross-compile with: cargo zigbuild --target <target>"
              echo ""
              echo -n "Available targets: ${pkgs.lib.strings.concatStringsSep ", " targets}"
              echo ""
            '';
          };

          packages = let
            cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
            mkRustPackage = toolchain:
              toolchain {
                pname = "${cargoToml.package.name}";
                inherit (cargoToml.package) version;

                src = pkgs.lib.cleanSource ./.;
                cargoLock.lockFile = ./Cargo.lock;

                inherit buildInputs;
                nativeBuildInputs = with pkgs; [pkg-config];
              };
          in {
            default = mkRustPackage pkgs.rustPlatform.buildRustPackage;
            static = mkRustPackage pkgs.pkgsStatic.rustPlatform.buildRustPackage;
          };

          pre-commit = {
            check.enable = true;
            settings = {
              hooks = {
                alejandra.enable = true;
                rustfmt.enable = true;
                deadnix.enable = true;
                statix.enable = true;
              };
            };
          };

          treefmt = {
            projectRootFile = "flake.nix";
            programs = {
              alejandra.enable = true;
              deadnix.enable = true;
              statix.enable = true;
              mdformat.enable = true;
            };
            settings = {
              on-unmatched = "info";
            };
          };
        };
    };
}
