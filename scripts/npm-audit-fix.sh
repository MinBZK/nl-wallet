#!/usr/bin/env bash

set -euo pipefail

SCRIPTS_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd -P)"

. "$SCRIPTS_DIR/configuration.sh"

# npm audit fix
while IFS= read -r -d '' lockfile; do
    echo "Fixing $lockfile..."
    cd "$(dirname "$lockfile")"
    npm audit fix || true
done < <(find "$BASE_DIR" -name package-lock.json -not -path '*/node_modules/*' -print0)

cd "$BASE_DIR"

branch='npm-audit-fix'

# Create MR if on CI and main
if [[ -n $CI && -n $(git status --porcelain) && $CI_COMMIT_BRANCH == "$CI_DEFAULT_BRANCH" ]]; then
    git config user.name 'Audit Fix'
    git config user.email "$AUDIT_FIX_GIT_EMAIL"
    git config commit.gpgsign 'true'
    # Unset include.path that sets url insteadOf with CI_GITLAB_TOKEN
    git config --unset include.path
    git config url.https://token:${AUDIT_FIX_GITLAB_TOKEN}@$CI_SERVER_HOST.insteadOf https://$CI_SERVER_HOST

    base64 -d <<< "$AUDIT_FIX_GPG_PRIVATE_KEY_BASE64" | gpg --import --quiet

    git switch -C "$branch"

    git add .
    git commit -m 'Run npm audit fix'

    # Force push if branch does not exist or exists but is different
    if ! git show-ref --quiet "origin/$branch" || [[ -n "$(git diff --shortstat "origin/$branch..$branch")" ]]; then
        git push -f origin "$branch"
    fi

    # Create MR if not already exists
    glab auth login --hostname "$CI_SERVER_HOST" --token "$AUDIT_FIX_GITLAB_TOKEN"
    if ! glab mr list -F json | jq -e --arg branch "$branch" 'any(select(.source_branch == $branch))' > /dev/null; then
        glab mr create -a "$AUDIT_FIX_REVIEWERS" -f -l security -y
    fi
fi
