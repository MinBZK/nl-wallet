#!/usr/bin/env bash
set -euxo pipefail

DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    libx11-6

sdkmanager --install "system-images;android-34;google_apis_playstore;x86_64"
avdmanager -s create avd --name phone --package "system-images;android-34;google_apis_playstore;x86_64"
