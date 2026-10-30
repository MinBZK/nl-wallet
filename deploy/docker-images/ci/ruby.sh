#!/usr/bin/env bash
set -euxo pipefail

DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
  bundler \
  build-essential \
  ruby-dev

# No RDoc
ruby_config_file="$(ruby -rrubygems -e'puts Gem::ConfigFile::SYSTEM_WIDE_CONFIG_FILE')"
cat > $ruby_config_file <<< "gem: --no-document"
gem install bundler-audit -v 0.9.3
