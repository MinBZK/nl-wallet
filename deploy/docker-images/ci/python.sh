#!/usr/bin/env bash
set -euxo pipefail

DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
  python3 python3-pip pipx git-filter-repo

sudo -E -H -u wallet -- sh -c 'pipx install --include-deps pip-audit==2.10.0 && rm -rf ~/.cache'
