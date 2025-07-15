ARG FROM_IMAGE_PREFIX
ARG TAG="latest"
FROM ${FROM_IMAGE_PREFIX}ci-node:${TAG}

# Playwright
COPY playwright.sh /tmp/
RUN /tmp/playwright.sh

# Cleanup
RUN rm -rf /tmp/*
