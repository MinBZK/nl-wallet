ARG FROM_IMAGE_PREFIX
ARG TAG="latest"
FROM ${FROM_IMAGE_PREFIX}ci-android:${TAG}

# Emulator
COPY android-emulator.sh /tmp/
RUN /tmp/android-emulator.sh

# Cleanup
RUN rm -rf /tmp/*
