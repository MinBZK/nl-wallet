set -euo pipefail

CUR_DIR="$(cd "$(dirname "$0")" && pwd)"

for file in "${CUR_DIR}"/unencrypted-gba-v-responses/*; do
    if [ -f "$file" ]; then
        "${CUR_DIR}"/gba_encrypt \
            --basename "$(basename "$file" .xml)" \
            --output /data \
            "$file"
    fi
done
