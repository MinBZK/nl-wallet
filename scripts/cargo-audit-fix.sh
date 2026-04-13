#!/usr/bin/env bash

set -euo pipefail

SCRIPTS_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd -P)"

. "$SCRIPTS_DIR/configuration.sh"

while IFS= read -r -d '' lockfile; do
    echo "Fixing $lockfile..."
    cd "$(dirname "$lockfile")"
    cargo audit fix || true
done < <(find "$BASE_DIR" -name Cargo.lock -not -path '*/target/*' -print0)

cd "$BASE_DIR"

# Create MR if on CI and main
if [[ -n $CI && -n $(git status --porcelain) && $CI_COMMIT_BRANCH == "$CI_DEFAULT_BRANCH" ]]; then
    exec "$BASE_DIR/deploy/bin/audit-fix-pr.sh" cargo-audit-fix "Run cargo audit fix"
fi
