ARG FROM_IMAGE_PREFIX
ARG TAG="latest"
FROM ${FROM_IMAGE_PREFIX}ci-rust:${TAG}

# Flutter
ENV FLUTTER_HOME=/opt/flutter
ENV PATH=${PATH}:${FLUTTER_HOME}/bin:/wallet/.pub-cache/bin
COPY flutter.sh /tmp/
RUN sudo -E -H -u wallet -- sh -c 'cd $HOME && /tmp/flutter.sh'
COPY rust-flutter.sh /tmp/
RUN /tmp/rust-flutter.sh

# Cleanup
RUN rm -rf /tmp/*
