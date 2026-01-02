#!/usr/bin/env bash
set -euo pipefail

CONTEXT_BASE="$( cd -- "$( dirname -- "$0" )" &> /dev/null && pwd -P )"

# Set IMAGE_NAME from $1
IMAGE_NAME="$1"
DOCKERFILE="${CONTEXT_BASE}/${IMAGE_NAME}.Dockerfile"
CONTEXT="${CONTEXT_BASE}/${IMAGE_NAME%%-*}"
FULL_IMAGE_PREFIX="${HARBOR_REGISTRY}/${HARBOR_PROJECT}/${IMAGE_PREFIX}"

# Tell us what image name we're using:
echo "Image to build is: \"$IMAGE_NAME\" with tag \"${IMAGE_TAG}\""

if [[ -e $DOCKERFILE ]]; then
    buildah build \
        --build-arg "DOCKER_HUB_PROXY=${HARBOR_REGISTRY}/docker-hub-proxy/" \
        --build-arg "FROM_IMAGE_PREFIX=${FULL_IMAGE_PREFIX}-" \
        --build-arg "TAG=${IMAGE_TAG}" \
        --file "${DOCKERFILE}" \
        --tag "${IMAGE_NAME}:${IMAGE_TAG}" \
        "${CONTEXT}"
    for HARBOR_REGISTRY in $HARBOR_REGISTRIES; do
        FULL_IMAGE="${HARBOR_REGISTRY}/${HARBOR_PROJECT}/${IMAGE_PREFIX}-${IMAGE_NAME}:${IMAGE_TAG}"
        buildah push ${IMAGE_NAME}:${IMAGE_TAG} "docker-daemon:${FULL_IMAGE}"
        buildah push ${IMAGE_NAME}:${IMAGE_TAG} "docker://${FULL_IMAGE}"
    done
else
    echo "ERROR: ${IMAGE_NAME}.Dockerfile does not exist!"
    exit 1
fi
