#!/usr/bin/env bash
#
# Usage: convert-gba-hc-converter-values.sh <asset-name> <path-to-files> > values.yaml
#
# Description:
#   Generates a YAML-formatted list of files from a directory or a single file.
#   The asset name is used as the key, and each file's content is the value.
#

set -euo pipefail

ASSET_NAME="${1:-}"
SCRIPT_PATH="${2:-}"

if [[ -z "$ASSET_NAME" ]] || [[ -z "$SCRIPT_PATH" ]]; then
  echo "Usage: $0 <asset-name> <path-to-files>" >&2
  exit 1
fi

if [[ ! -e "$SCRIPT_PATH" ]]; then
  echo "Error: Path '$SCRIPT_PATH' does not exist." >&2
  exit 2
fi

echo "${ASSET_NAME}:"
if [[ -f "$SCRIPT_PATH" ]]; then
  filename="$(basename "$SCRIPT_PATH")"
  echo "  $filename: |"
  # Indent each line by 4 spaces
  sed 's/^/    /' "$SCRIPT_PATH"
elif [[ -d "$SCRIPT_PATH" ]]; then
  for file in "$SCRIPT_PATH"/*; do
    [[ -f "$file" ]] || continue
    filename="$(basename "$file")"
    echo "  $filename: |"
    sed 's/^/    /' "$file"
  done
fi
