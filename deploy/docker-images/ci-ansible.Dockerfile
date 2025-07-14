ARG FROM_IMAGE_PREFIX
ARG TAG="latest"
FROM ${FROM_IMAGE_PREFIX}ci-base:${TAG}

RUN sudo -E -H -u wallet -- sh -c 'pipx install --include-deps ansible==11.7.0'
