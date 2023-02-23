#!/usr/bin/env bash

if [ $# -lt 1 ]; then
    echo "No native language provided"
    exit 1
fi

NATIVE_LANGUAGE=$1
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

cargo run --manifest-path "$SCRIPT_DIR/Cargo.toml" --no-default-features --features uniffi-bindgen --bin uniffi-bindgen generate "$SCRIPT_DIR/udl/hw_keystore.udl" --language "$NATIVE_LANGUAGE" --out-dir "$SCRIPT_DIR/$NATIVE_LANGUAGE"
