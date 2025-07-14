#!/usr/bin/env bash
set -euxo pipefail

wget --no-hsts -q -O /usr/local/bin/osv-scanner https://github.com/google/osv-scanner/releases/download/v1.9.2/osv-scanner_linux_amd64
echo "d6af4b67fa5de658598bd2d445efb99e90d1734b3146962418719c4350ecb74b  /usr/local/bin/osv-scanner" | sha256sum -c
chmod +x /usr/local/bin/osv-scanner
