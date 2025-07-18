ARG FROM_IMAGE_PREFIX
ARG TAG="latest"
FROM ${FROM_IMAGE_PREFIX}ci-base:${TAG}

# Node
COPY node.sh /tmp/
RUN /tmp/node.sh

# Cleanup
RUN rm -rf /tmp/*
