#!/usr/bin/env bash
set -euxo pipefail

wget --no-hsts -q -O ./setup.sh https://deb.nodesource.com/setup_22.x
echo "c61e58b2284efea4746ffbdcb4d4080f58fa7a31fb0060d2b024eb3d2e95572d  setup.sh" | sha256sum -c

chmod +x ./setup.sh
./setup.sh
rm ./setup.sh

DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    nodejs

npm --version
node --version
