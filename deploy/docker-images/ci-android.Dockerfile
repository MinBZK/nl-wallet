ARG FROM_IMAGE_PREFIX
ARG TAG="latest"
FROM ${FROM_IMAGE_PREFIX}ci-flutter:${TAG}

# Android
ENV ANDROID_HOME=/opt/android-sdk
ENV ANDROID_NDK_HOME=/opt/android-sdk/ndk/28.1.13356709
ENV PATH=${PATH}:${ANDROID_HOME}/cmdline-tools/latest/bin:${ANDROID_HOME}/emulator:${ANDROID_HOME}/platform-tools:${ANDROID_NDK_HOME}
COPY android.sh android-ndk* /tmp/
RUN /tmp/android.sh
RUN /tmp/android-ndk.sh
RUN /tmp/android-ndk-21.sh
COPY rust-android.sh /tmp
RUN sudo -E -H -u wallet -- sh -c 'cd $HOME && /tmp/rust-android.sh'

# Cleanup
RUN rm -rf /tmp/*
