ARG FROM_IMAGE_PREFIX
ARG TAG="latest"
FROM ${FROM_IMAGE_PREFIX}ci-base:${TAG}

# Node
COPY node.sh /dockerfiles/
RUN /dockerfiles/node.sh
