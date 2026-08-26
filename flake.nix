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

            flutter
            android-tools
          ];

          shellHook = ''
            # Tools installed through `cargo install`, such as `flutter_rust_bridge_codegen` and `sea-orm-cli`, end up
            # here. Cargo locates its own subcommands (`cargo nextest`) in this directory regardless of `PATH`, but
            # standalone binaries are only found when it is actually on `PATH`.
            export PATH="''${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"

            # The Dart pub workspace at the root of this repository requires the Flutter SDK in its `environment`.
            # Plain `dart pub` can only satisfy that constraint when `FLUTTER_ROOT` is set, which `flutter pub` does
            # implicitly but `dart` does not. Without this, `flutter_rust_bridge_codegen` fails while running
            # `dart run ffigen`.
            export FLUTTER_ROOT="$(dirname "$(dirname "$(readlink -f "$(command -v flutter)")")")"
          '';
        };
      });
    };
}
