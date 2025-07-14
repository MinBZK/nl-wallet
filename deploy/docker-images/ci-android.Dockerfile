ARG FROM_IMAGE_PREFIX
ARG TAG="latest"
FROM ${FROM_IMAGE_PREFIX}ci-flutter:${TAG}

# Android
ENV ANDROID_HOME=/opt/android-sdk
ENV ANDROID_NDK_HOME=/opt/android-sdk/ndk/28.1.13356709
ENV PATH=${PATH}:${ANDROID_HOME}/cmdline-tools/latest/bin:${ANDROID_HOME}/emulator:${ANDROID_HOME}/platform-tools:${ANDROID_NDK_HOME}
COPY android.sh android-ndk* /dockerfiles/
RUN /dockerfiles/android.sh
RUN /dockerfiles/android-ndk.sh
RUN /dockerfiles/android-ndk-21.sh
COPY rust-android.sh /dockerfiles
RUN sudo -E -H -u wallet -- sh -c 'cd $HOME && /dockerfiles/rust-android.sh'
