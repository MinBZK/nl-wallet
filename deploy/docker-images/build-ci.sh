#!/usr/bin/env bash
set -euo pipefail

CONTEXT_BASE="$( cd -- "$( dirname -- "$0" )" &> /dev/null && pwd -P )"

# Set IMAGE_NAME from $1
IMAGE_NAME="$1"
DOCKERFILE="${CONTEXT_BASE}/${IMAGE_NAME}.Dockerfile"
CONTEXT="${CONTEXT_BASE}/${IMAGE_NAME%%-*}"
FROM_IMAGE_PREFIX="${HARBOR_REGISTRY}/${HARBOR_PROJECT}/${IMAGE_PREFIX}-"

# Tell us what image name we're using:
echo "Image to build is: \"$IMAGE_NAME\" with tag \"${IMAGE_TAG}\""

if [[ -e $DOCKERFILE ]]; then
    # Prefetch previous layers from docker daemon of host to speed up
    for ID in $(awk '/^FROM \$\{FROM_IMAGE_PREFIX\}/ {
        gsub(/\$\{FROM_IMAGE_PREFIX}/, "'"${FROM_IMAGE_PREFIX}"'", $2);
        gsub(/\$\{TAG}/, "'"${IMAGE_TAG}"'", $2);
        print $2;
    }' "${DOCKERFILE}"); do
        echo "Prefetching $ID"
        buildah pull "docker-daemon:${ID}" || true
    done

    echo "Building"
    buildah build \
        --build-arg "DOCKER_HUB_PROXY=${HARBOR_REGISTRY}/docker-hub-proxy/" \
        --build-arg "FROM_IMAGE_PREFIX=${FROM_IMAGE_PREFIX}" \
        --build-arg "TAG=${IMAGE_TAG}" \
        --file "${DOCKERFILE}" \
        --tag "${IMAGE_NAME}:${IMAGE_TAG}" \
        "${CONTEXT}"
    for HARBOR_REGISTRY in $HARBOR_REGISTRIES; do
        FULL_IMAGE="${HARBOR_REGISTRY}/${HARBOR_PROJECT}/${IMAGE_PREFIX}-${IMAGE_NAME}:${IMAGE_TAG}"

        # Push to docker daemon of host to speed up next use
        echo "Pushing for $HARBOR_REGISTRY to docker-daemon"
        buildah push ${IMAGE_NAME}:${IMAGE_TAG} "docker-daemon:${FULL_IMAGE}"

        echo "Pushing for $HARBOR_REGISTRY to registry"
        buildah push ${IMAGE_NAME}:${IMAGE_TAG} "docker://${FULL_IMAGE}"
    done
else
    echo "ERROR: ${IMAGE_NAME}.Dockerfile does not exist!"
    exit 1
fi
