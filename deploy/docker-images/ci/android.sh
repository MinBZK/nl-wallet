#!/usr/bin/env bash
set -euxo pipefail

wget --no-hsts -q -O sdk.zip https://dl.google.com/android/repository/commandlinetools-linux-13114758_latest.zip
echo "7ec965280a073311c339e571cd5de778b9975026cfcbe79f2b1cdcb1e15317ee sdk.zip" | sha256sum -c

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
  "build-tools;35.0.0" \
  "build-tools;36.0.0" \
  "cmake;3.22.1" \
  "cmdline-tools;19.0" \
  "platforms;android-31" \
  "platforms;android-33" \
  "platforms;android-34" \
  "platforms;android-35" \
  "platforms;android-36" \
  "platform-tools"

rm -rf ~/.android
