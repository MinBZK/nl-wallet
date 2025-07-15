ARG FROM_IMAGE_PREFIX
ARG TAG="latest"
FROM ${FROM_IMAGE_PREFIX}ci-node:${TAG}

# SoftHSM
COPY softhsm.sh /tmp/
RUN /tmp/softhsm.sh

# Rust
COPY rust-root.sh /tmp/
RUN /tmp/rust-root.sh
ENV PATH=${PATH}:/wallet/.cargo/bin
COPY rust-user.sh /tmp/
RUN sudo -E -H -u wallet -- sh -c 'cd $HOME && /tmp/rust-user.sh'

# Cleanup
RUN rm -rf /tmp/*
