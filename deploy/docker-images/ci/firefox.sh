#!/usr/bin/env bash
set -euxo pipefail

DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
  firefox-esr

wget --no-hsts -q -O geckodriver.tar.gz \
  https://github.com/mozilla/geckodriver/releases/download/v0.36.0/geckodriver-v0.36.0-linux64.tar.gz
echo "0bde38707eb0a686a20c6bd50f4adcc7d60d4f73c60eb83ee9e0db8f65823e04  geckodriver.tar.gz" | sha256sum -c

tar -xf geckodriver.tar.gz -C /usr/local/bin geckodriver
chmod +x /usr/local/bin/geckodriver
rm geckodriver.tar.gz
