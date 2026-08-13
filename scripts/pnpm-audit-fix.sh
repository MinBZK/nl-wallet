#!/usr/bin/env bash

set -euo pipefail

SCRIPTS_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd -P)"

. "$SCRIPTS_DIR/configuration.sh"

while IFS= read -r -d '' lockfile; do
    echo "Fixing $lockfile..."
    cd "$(dirname "$lockfile")"
    pnpm audit --fix update || true
done < <(find "$BASE_DIR" -name pnpm-lock.yaml -not -path '*/node_modules/*' -print0)

cd "$BASE_DIR"

# Create MR if on CI and main
if [[ -n $CI && -n $(git status --porcelain) && $CI_COMMIT_BRANCH == "$CI_DEFAULT_BRANCH" ]]; then
    exec "$BASE_DIR/deploy/bin/audit-fix-pr.sh" pnpm-audit-fix "Run pnpm audit fix"
fi
