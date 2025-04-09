#!/usr/bin/env bash

set -eu

source "$(cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd)/store-functions.sh"

check

clean() {
    if [[ -f ${zipfile:-} ]]; then
        rm $zipfile
    fi
}
trap clean EXIT

target="$1"
source="$(mktemp).zip"
zip -j $source "${@:2:$#}"
store "$source" "$target"
