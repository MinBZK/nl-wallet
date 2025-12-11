#!/usr/bin/env bash
set -euxo pipefail

sdkmanager --install "ndk;28.2.13676358"

rm -rf ~/.android
