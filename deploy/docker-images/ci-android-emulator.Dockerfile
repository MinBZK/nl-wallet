ARG FROM_IMAGE_PREFIX
ARG TAG="latest"
FROM ${FROM_IMAGE_PREFIX}ci-android:${TAG}

# Emulator
COPY android-emulator.sh /dockerfiles/
RUN /dockerfiles/android-emulator.sh
