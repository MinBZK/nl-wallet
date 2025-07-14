#!/usr/bin/env bash
set -euxo pipefail

DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
  bundler \
  build-essential \
  ruby-dev

# No RDoc
cat > $(ruby -rrubygems -e'puts Gem::ConfigFile::SYSTEM_WIDE_CONFIG_FILE') <<< "gem: --no-document"
gem install bundler-audit -v 0.9.2
