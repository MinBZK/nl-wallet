#!/usr/bin/env bash
set -euxo pipefail

wget --no-hsts -q -O zap.tar.gz https://github.com/zaproxy/zaproxy/releases/download/v2.16.0/ZAP_2.16.0_Linux.tar.gz
echo "a0779509e702ec53d41074eaa0ce41f2a964a822aa5be0380255a482e2e7fe8d  zap.tar.gz" | sha256sum -c

tar -xf zap.tar.gz
mv -T ZAP_* $ZAP_HOME
rm  -rf zap.tar.gz

# Install all add-ons
"$ZAP_HOME/zap.sh" -cmd -addoninstallall
