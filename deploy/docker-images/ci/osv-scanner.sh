#!/usr/bin/env bash
set -euxo pipefail

wget --no-hsts -q -O /usr/local/bin/osv-scanner https://github.com/google/osv-scanner/releases/download/v2.7.1/osv-scanner_linux_amd64
echo "fb7dab19a8bd9bbe5c3189f09d4b7ee1820c9553a23bac202648a56efd2f6ee8  /usr/local/bin/osv-scanner" | sha256sum -c
chmod +x /usr/local/bin/osv-scanner
