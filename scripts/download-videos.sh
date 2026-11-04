#!/usr/bin/env bash

SCRIPTS_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd -P)"
export SCRIPTS_DIR

source "${SCRIPTS_DIR}/configuration.sh"

set -euo pipefail

VIDEOS_URL="${VIDEOS_URL:-https://static.example.com/videos}"
VIDEOS_DIR="${STATIC_SERVER_DIR}/videos"
WALLET_ASSETS="${BASE_DIR}/wallet_app/lib/src/wallet_assets.dart"

languages=$(grep 'supported_video_language_codes' "${WALLET_ASSETS}" | grep -oE "'[a-z]{2}'" | tr -d "'")
if [[ $# -gt 0 ]]; then
    slugs="$*"
else
    slugs=$(sed -n '/video_slugs = \[/,/\];/p' "${WALLET_ASSETS}" | grep -oE "'[a-z0-9-]+'" | tr -d "'")
fi

mkdir -p "${VIDEOS_DIR}"
for language in ${languages}; do
    for slug in ${slugs}; do
        for extension in mp4 srt; do
            file="${language}-${slug}.${extension}"
            if [[ -f "${VIDEOS_DIR}/${file}" ]]; then
                echo "Skipping ${file}, already downloaded"
                continue
            fi
            echo "Downloading ${file}"
            curl  -fsSL -o "${VIDEOS_DIR}/${file}.part" "${VIDEOS_URL}/${file}"
            mv "${VIDEOS_DIR}/${file}.part" "${VIDEOS_DIR}/${file}"
        done
    done
done
