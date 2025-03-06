check() {
    if [[ -z ${CI:-} ]]; then
        # Set ourselves for local testing
        if [[ -z ${GIT_COMMIT_SHA:-} ]]; then
            GIT_COMMIT_SHA=$(git rev-parse HEAD)
        fi
    else
        # Only allow on main branch
        [[ ${CI_COMMIT_BRANCH:-} == "$CI_DEFAULT_BRANCH" ]] || exit 0

        # Only store artifacts in after_script if successful
        [[ $CI_JOB_STATUS == "running" || $CI_JOB_STATUS == "success" ]] || exit 0

        GIT_COMMIT_SHA=$CI_COMMIT_SHA
    fi
}

store() {
    local source=$1
    local target=$2

    if [[ $target =~ \/$ ]]; then
        target="${target}$(basename "$source")"
    fi

    local previous=$(mc stat $target | awk 'BEGIN { FS=": " } /^  X-Amz-Meta-Git-Commit-Sha/ { print $2 }')
    if [[ -z $previous ]] || git merge-base --is-ancestor $previous $GIT_COMMIT_SHA; then
        echo "Storing $source"
        mc cp --attr "Git-Commit-Sha=$GIT_COMMIT_SHA" $source $target
    else
        echo "Skipping $source"
    fi
}
