#!/usr/bin/env bash
set -euxo pipefail

# install older ndk for root_jailbreak_sniffer plugin.
sdkmanager --install "ndk;21.4.7075529"

rm -rf ~/.android
