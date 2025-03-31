#!/usr/bin/env bash

if [ $# -lt 2 ]; then
    echo "No module and native language provided"
    exit 1
fi

MODULE_NAME=$1
NATIVE_LANGUAGE=$2
CONFIG_FILE="uniffi.toml"

SCRIPT_DIR=$(dirname $(realpath ${BASH_SOURCE[0]}))

cargo run --manifest-path "$SCRIPT_DIR/../../uniffi-bindgen/Cargo.toml" generate "$SCRIPT_DIR/udl/$MODULE_NAME.udl" --language "$NATIVE_LANGUAGE" --out-dir "$SCRIPT_DIR/$NATIVE_LANGUAGE" --config "$SCRIPT_DIR/$CONFIG_FILE" --no-format
