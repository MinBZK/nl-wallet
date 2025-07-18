#!/usr/bin/env bash
set -euxo pipefail

DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    libicu-dev

wget --no-hsts -q -O /usr/local/bin/cyclonedx https://github.com/CycloneDX/cyclonedx-cli/releases/download/v0.27.2/cyclonedx-linux-x64
echo "5e1595542a6367378a3944bbd3008caab3de65d572345361d3b9597b1dbbaaa0  /usr/local/bin/cyclonedx" | sha256sum -c
chmod +x /usr/local/bin/cyclonedx

# Rust
sudo -E -H -u wallet -- sh -c 'cd $HOME && cargo install cargo-cyclonedx --version 0.5.7'

# Ruby
gem install cyclonedx-cocoapods -v 2.0.0

# Node
npm install --location=global @cyclonedx/cdxgen@v11.1.2 @cyclonedx/cyclonedx-npm@v1.20.0
rm -rf ~/.npm /tmp/node-compile-cache
