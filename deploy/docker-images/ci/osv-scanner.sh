#!/usr/bin/env bash
set -euxo pipefail

wget --no-hsts -q -O /usr/local/bin/osv-scanner https://github.com/google/osv-scanner/releases/download/v2.3.3/osv-scanner_linux_amd64
echo "777b4bb7ddd10bdcc8a1aa398d37d05e91e866e7586f9cff3fca2f72b8153033  /usr/local/bin/osv-scanner" | sha256sum -c
chmod +x /usr/local/bin/osv-scanner
