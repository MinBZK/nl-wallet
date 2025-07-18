#!/usr/bin/env bash
set -euxo pipefail

DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
  python3 python3-pip pipx

# Download from GitHub as Debian version is too old
wget --no-hsts -q -O /usr/local/bin/git-filter-repo https://github.com/newren/git-filter-repo/raw/refs/tags/v2.47.0/git-filter-repo
echo "67447413e273fc76809289111748870b6f6072f08b17efe94863a92d810b7d94  /usr/local/bin/git-filter-repo" | sha256sum -c
chmod +x /usr/local/bin/git-filter-repo
