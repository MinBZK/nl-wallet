ARG FROM_IMAGE_PREFIX
ARG TAG="latest"
FROM ${FROM_IMAGE_PREFIX}ci-node:${TAG}

# SoftHSM
COPY softhsm.sh /dockerfiles/
RUN /dockerfiles/softhsm.sh

# Rust
COPY rust-root.sh /dockerfiles/
RUN /dockerfiles/rust-root.sh
ENV PATH=${PATH}:/wallet/.cargo/bin
COPY rust-user.sh /dockerfiles/
RUN sudo -E -H -u wallet -- sh -c 'cd $HOME && /dockerfiles/rust-user.sh'
