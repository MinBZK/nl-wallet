#!/usr/bin/env bash
set -euxo pipefail

wget --no-hsts -q -O ./setup.sh https://deb.nodesource.com/setup_24.x
echo "872150825071bb403b6e210c66d9487a791047d0299026de429eb1f626fd969f  setup.sh" | sha256sum -c

chmod +x ./setup.sh
./setup.sh
rm ./setup.sh

DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    nodejs

npm --version
node --version
