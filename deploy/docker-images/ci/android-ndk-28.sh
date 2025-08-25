#!/usr/bin/env bash
set -euxo pipefail

sdkmanager --install "ndk;28.1.13356709"

rm -rf ~/.android
