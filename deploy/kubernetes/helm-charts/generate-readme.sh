#!/usr/bin/env bash

set -euo pipefail

BASE_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd -P)"

if command -v readme-generator > /dev/null; then
    GENERATOR='readme-generator'
elif command -v npx > /dev/null; then
    GENERATOR='npx @bitnami/readme-generator-for-helm'
else
    >&2 echo 'ERROR: Cannot find readme-generator or npx'
    exit 1
fi

for values_file in $(find "$BASE_DIR" -type f -name 'values.yaml'); do
    echo "$(basename "$(dirname "$(dirname "$values_file")")")"
    $GENERATOR -r "$(dirname "$values_file")/README.md" -v "$values_file"
done
