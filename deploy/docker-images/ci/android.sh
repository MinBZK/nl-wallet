#!/usr/bin/env bash
set -euxo pipefail

wget --no-hsts -q -O sdk.zip https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip
echo "2d2d50857e4eb553af5a6dc3ad507a17adf43d115264b1afc116f95c92e5e258 sdk.zip" | sha256sum -c

unzip -q sdk.zip
rm sdk.zip

mkdir -p $ANDROID_HOME/cmdline-tools
mv cmdline-tools $ANDROID_HOME/cmdline-tools/latest

# Packages
set +o pipefail
yes | sdkmanager --licenses
set -o pipefail

# "ndk should be installed separately due to size.
sdkmanager --install \
  "build-tools;34.0.0" \
  "cmake;3.22.1" \
  "cmdline-tools;17.0" \
  "platforms;android-31" \
  "platforms;android-33" \
  "platforms;android-34" \
  "platforms;android-35" \
  "platform-tools"

rm -rf ~/.android
