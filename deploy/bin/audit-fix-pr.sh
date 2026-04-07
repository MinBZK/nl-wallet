#!/usr/bin/env bash

set -euo pipefail

branch="${1:-}"
message="${2:-}"

if [[ -z $branch ]]; then
    >&2 echo "ERROR: Specify branch"
    exit 1
fi

if [[ -z $message ]]; then
    >&2 echo "ERROR: Specify commit message"
    exit 1
fi

git config user.name 'Audit Fix'
git config user.email "$AUDIT_FIX_GIT_EMAIL"
git config commit.gpgsign 'true'
# Unset include.path that sets url insteadOf with CI_GITLAB_TOKEN
git config --unset include.path
git config url.https://token:${AUDIT_FIX_GITLAB_TOKEN}@$CI_SERVER_HOST.insteadOf https://$CI_SERVER_HOST

base64 -d <<< "$AUDIT_FIX_GPG_PRIVATE_KEY_BASE64" | gpg --import --quiet

git switch -C "$branch"

git add .
git commit -m "$message"

# Force push if branch does not exist or exists but is different
if ! git show-ref --quiet "origin/$branch" || [[ -n "$(git diff --shortstat "origin/$branch..$branch")" ]]; then
    git push -f origin "$branch"
fi

# Create MR if not already exists
glab auth login --hostname "$CI_SERVER_HOST" --token "$AUDIT_FIX_GITLAB_TOKEN"
if ! glab mr list -F json | jq -e --arg branch "$branch" 'any(select(.source_branch == $branch))' > /dev/null; then
    glab mr create -a "$AUDIT_FIX_REVIEWERS" -f -l security -y
fi
