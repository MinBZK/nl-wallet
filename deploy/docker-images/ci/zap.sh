#!/usr/bin/env bash
set -euxo pipefail

wget --no-hsts -q -O zap.tar.gz https://github.com/zaproxy/zaproxy/releases/download/v2.17.0/ZAP_2.17.0_Linux.tar.gz
echo "efe799aaa3627db683b43f00c9c210aea0b75c00cc8f0a0f0434d12bb3ddde5a  zap.tar.gz" | sha256sum -c

tar -xf zap.tar.gz
mv -T ZAP_* $ZAP_HOME
rm  -rf zap.tar.gz

# Install all add-ons
$ZAP_HOME/zap.sh -cmd -addoninstallall
