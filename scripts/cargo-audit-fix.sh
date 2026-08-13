#!/usr/bin/env bash

set -euo pipefail

SCRIPTS_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd -P)"

. "$SCRIPTS_DIR/configuration.sh"

audit_count() {
    # cargo audit exits with non-zero if vulnerabilties are found which causes script to fail due to pipefail
    audit=$(cargo audit --json)
    jq '(.vulnerabilities.list + (.warnings | to_entries | map(.value) | flatten(1))) | length' <<< "$audit"
}

while IFS= read -r -d '' lockfile; do
    echo "Fixing $lockfile..."

    cd "$(dirname "$lockfile")"
    before=$(audit_count)
    cargo audit fix || true
    after=$(audit_count)

    # Revert changes if the vulnerabilties did not decrease
    # Can happen due to: https://github.com/rust-lang/cargo/issues/14115
    if [[ $before -le $after ]]; then
        echo "No fixes found"
        git checkout $lockfile
    fi
done < <(find "$BASE_DIR" -name Cargo.lock -not -path '*/target/*' -print0)

cd "$BASE_DIR"

# Create MR if on CI and main
if [[ -n ${CI:-} && -n $(git status --porcelain) && $CI_COMMIT_BRANCH == "$CI_DEFAULT_BRANCH" ]]; then
    exec "$BASE_DIR/deploy/bin/audit-fix-pr.sh" cargo-audit-fix "Run cargo audit fix"
fi
