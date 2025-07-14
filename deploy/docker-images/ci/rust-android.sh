#!/usr/bin/env bash
set -euxo pipefail

rustup target add armv7-linux-androideabi aarch64-linux-android x86_64-linux-android i686-linux-android
cargo install cargo-ndk --version 3.5.4
