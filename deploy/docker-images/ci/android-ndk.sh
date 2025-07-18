#!/usr/bin/env bash
set -euxo pipefail

# ndk should be installed separately due to size.
sdkmanager --install "ndk;$(basename $ANDROID_NDK_HOME)"

rm -rf ~/.android
