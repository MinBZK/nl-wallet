#!/usr/bin/env bash
set -euxo pipefail

wget --no-hsts -q -O ./setup.sh https://deb.nodesource.com/setup_24.x
echo "6e3d580f5bd7ccf2aa1e8df8d35c60d78e873c3ff8beb282c9bebd914904ad72  setup.sh" | sha256sum -c

chmod +x ./setup.sh
./setup.sh
rm ./setup.sh

DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    nodejs

npm --version
node --version
