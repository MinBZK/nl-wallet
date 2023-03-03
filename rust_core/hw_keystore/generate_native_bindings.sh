#!/usr/bin/env bash

if [ $# -lt 1 ]; then
    echo "No native language provided"
    exit 1
fi

NATIVE_LANGUAGE=$1
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

"$HOME/.cargo/bin/cargo" run --manifest-path "$SCRIPT_DIR/../uniffi-bindgen/Cargo.toml" generate "$SCRIPT_DIR/udl/hw_keystore.udl" --language "$NATIVE_LANGUAGE" --out-dir "$SCRIPT_DIR/$NATIVE_LANGUAGE"
