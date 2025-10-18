#!/usr/bin/env bash
set -euxo pipefail

wget --no-hsts -q -O ./setup.sh https://deb.nodesource.com/setup_22.x
echo "02983a54150ea7e5072bbb06b655be7a8c628e4556e85fb0942f719ec50a1d3a  setup.sh" | sha256sum -c

chmod +x ./setup.sh
./setup.sh
rm ./setup.sh

DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    nodejs

npm --version
node --version
