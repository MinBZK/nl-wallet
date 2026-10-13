{
  description = "A Nix-flake-based NL Wallet development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forEachSupportedSystem = f: nixpkgs.lib.genAttrs supportedSystems (system: f {
        pkgs = import nixpkgs {
          inherit system;
        };
      });
    in
    {
      devShells = forEachSupportedSystem ({ pkgs }: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            openssl
            pkg-config

            jq
            envsubst
            softhsm
            gnutls
            xxd
            nodejs
            pnpm
            prettier

            fvm
            android-tools
          ];

          shellHook = ''
            # Tools installed through `cargo install`, such as `flutter_rust_bridge_codegen` and `sea-orm-cli`, end up
            # here. Cargo locates its own subcommands (`cargo nextest`) in this directory regardless of `PATH`, but
            # standalone binaries are only found when it is actually on `PATH`.
            export PATH="''${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"

            # Flutter is managed by FVM at the version pinned in `wallet_app/.fvmrc` (run `fvm install` in `wallet_app`
            # once; on NixOS this requires `programs.nix-ld`). The nixpkgs Flutter cannot be used: it symlinks its Dart
            # SDK from a separate store path, so `build_runner` does not find `dart:ui` next to it and generates broken
            # code.
            flutter_version="$(jq -r .flutter "$(git rev-parse --show-toplevel)/wallet_app/.fvmrc")"
            flutter_sdk="''${FVM_CACHE_PATH:-$HOME/fvm}/versions/$flutter_version"
            if [ -x "$flutter_sdk/bin/flutter" ]; then
              export PATH="$flutter_sdk/bin:$PATH"

              # The Dart pub workspace at the root of this repository requires the Flutter SDK in its `environment`.
              # Plain `dart pub` can only satisfy that constraint when `FLUTTER_ROOT` is set, which `flutter pub` does
              # implicitly but `dart` does not. Without this, `flutter_rust_bridge_codegen` fails while running
              # `dart run ffigen`.
              export FLUTTER_ROOT="$flutter_sdk"
            else
              echo "Flutter $flutter_version not found, run \`fvm install\` in wallet_app" >&2
            fi
          '';
        };
      });
    };
}
