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

find "$BASE_DIR" -type f -name 'values.yaml' -print0 |
  while IFS= read -r -d '' values_file; do
    chart_dir="$(dirname -- "$values_file")"
    basename -- "$chart_dir"
    "$GENERATOR" -r "$chart_dir/README.md" -v "$values_file"
  done
