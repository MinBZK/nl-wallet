#!/usr/bin/env bash
set -euxo pipefail

sdkmanager --install "ndk;27.0.12077973"

rm -rf ~/.android
