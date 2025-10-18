#!/usr/bin/env bash

set -euo pipefail

SCRIPTS_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd -P)"
BASE_DIR="$(dirname "${SCRIPTS_DIR}")"

cd "$BASE_DIR/wallet_app"

flutter pub get --enforce-lockfile
if [[ -z ${CI:-} ]]; then
    # Also flutter pub get sub packages on local machines because it does not
    # automatically update transitively. On CI this is not needed as there is
    # no cache yet.
    for package in packages/wallet_*; do
        if [[ -d $package ]]; then
            cd $package
            flutter pub get
            cd ../..
        fi
    done
fi

flutter_rust_bridge_codegen generate --config-file flutter_rust_bridge.yaml
flutter pub run build_runner build --delete-conflicting-outputs

# `flutter_rust_bridge_codegen` already formats the generated code, but it apparently doesn't match our style
dart format . --line-length 120
