#!/usr/bin/env bash

set -euo pipefail

SCRIPTS_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd -P)"
BASE_DIR="$(dirname -- "${SCRIPTS_DIR}")"

if [[ -z $1 ]]; then
    >&2 echo "ERROR: specify artifact zip file"
    exit 1
fi

# Create temp dir for artifacts
artifact_dir="$(mktemp -d)"
clean() {
    if [[ -d ${artifact_dir:-} ]]; then
        rm -rf $artifact_dir
    fi
}
trap clean EXIT

# Unzip artifact zip into tempdir
unzip -q -d "$artifact_dir" "$1"

# Create associative array of masters and files
declare -A masters
while read -ra line; do
    md5="${line[0]}"
    masters["$md5"]="${line[@]:1}"
done < <(find "${BASE_DIR}/wallet_app/test/src" -path '*/goldens/*.png' -print0 | xargs -0 md5sum)

# Copy all generated images on pipeline as golden
while read -ra line; do
    md5="${line[0]}"
    source="${line[@]:1}"

    base="${source#"${artifact_dir}/"}"
    target="${masters["$md5"]:-}"
    if [[ -z $target ]]; then
        >&2 echo "Cannot find target for $base"
        continue
    fi

    source="${source%_masterImage.png}_testImage.png"
    if [[ -e $source ]]; then
        mv "$source" "$target"
    else
        >&2 echo "Cannot find source for $base"
    fi
done < <(find "$artifact_dir" -type f -name '*_masterImage.png' -print0 | xargs -0 md5sum)
