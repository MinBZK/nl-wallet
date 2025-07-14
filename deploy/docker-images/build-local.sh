#!/usr/bin/env bash
set -euo pipefail

CONTEXT_BASE=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

if [[ $# -eq 0 ]]; then
    >&2 echo "ERROR: Specify the images to build"
    exit 1
fi

for name in $@; do
    file="${CONTEXT_BASE}/${name}.Dockerfile"
    if [[ ! -f $file ]]; then
        >&2 echo "ERROR: $file does not exixt"
        exit 1
    fi
    docker build \
        --build-arg "FROM_IMAGE_PREFIX=nl-wallet-" \
        --file $file \
        --tag "nl-wallet-${name}:latest" \
        "${CONTEXT_BASE}/${name%%-*}"
done
