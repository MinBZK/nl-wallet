#!/usr/bin/env bash
set -euxo pipefail

wget --no-hsts -q -O zap.tar.gz https://github.com/zaproxy/zaproxy/releases/download/v2.16.1/ZAP_2.16.1_Linux.tar.gz
echo "5b2eb8319b085121a6e8ad50d69d67dbef8c867166f71a937bfc888d247a2ac1  zap.tar.gz" | sha256sum -c

tar -xf zap.tar.gz
mv -T ZAP_* $ZAP_HOME
rm  -rf zap.tar.gz

# Install all add-ons
$ZAP_HOME/zap.sh -cmd -addoninstallall
