#!/usr/bin/env bash
set -euxo pipefail

# LLVM is required by the ffigen Dart package, which in turn
# is used by flutter-rust-bridge to generate Dart code.
DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
  libclang-dev
