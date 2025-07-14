ARG FROM_IMAGE_PREFIX
ARG TAG="latest"
FROM ${FROM_IMAGE_PREFIX}ci-node:${TAG}

COPY playwright.sh /dockerfiles/
RUN /dockerfiles/playwright.sh
