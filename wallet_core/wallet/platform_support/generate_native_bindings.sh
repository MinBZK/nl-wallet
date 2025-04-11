#!/usr/bin/env bash

set -euo pipefail

if [[ $# -lt 2 ]]; then
    >&2 echo "ERROR: No native language and module provided"
    >&2 echo "Usage: $(basename ${BASH_SOURCE[0]}) NATIVE_LANGUAGE MODULE ..."
    exit 1
fi

CONFIG_FILE="uniffi.toml"
SCRIPT_DIR=$(dirname $(realpath ${BASH_SOURCE[0]}))

NATIVE_LANGUAGE=$1
for MODULE in ${@:2:$#}; do
    MODULE=${MODULE%.udl}
    cargo run --manifest-path "$SCRIPT_DIR/../../uniffi-bindgen/Cargo.toml" generate "$SCRIPT_DIR/udl/$MODULE.udl" \
              --language "$NATIVE_LANGUAGE" --out-dir "$SCRIPT_DIR/$NATIVE_LANGUAGE" --config "$SCRIPT_DIR/$CONFIG_FILE" --no-format
done
