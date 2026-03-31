#!/usr/bin/env bash
set -euxo pipefail

DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    libicu-dev

wget --no-hsts -q -O /usr/local/bin/cyclonedx https://github.com/CycloneDX/cyclonedx-cli/releases/download/v0.30.0/cyclonedx-linux-x64
echo "f89876326620f5fc78a9b27cc1af57d6ed13d019aab87490e1246a44a910babb  /usr/local/bin/cyclonedx" | sha256sum -c
chmod +x /usr/local/bin/cyclonedx

# Rust
sudo -E -H -u wallet -- sh -c 'cd $HOME && cargo install cargo-cyclonedx --locked --version 0.5.9'

# Ruby
gem install cyclonedx-cocoapods -v 2.0.1

# Node
npm install --ignore-scripts --location=global @cyclonedx/cdxgen@v11.1.2 @cyclonedx/cyclonedx-npm@v3.0.0
rm -rf ~/.npm /tmp/node-compile-cache
