#!/usr/bin/env bash
set -euxo pipefail

wget --no-hsts -q -O sonar-scanner-cli.zip https://binaries.sonarsource.com/Distribution/sonar-scanner-cli/sonar-scanner-cli-7.3.0.5189-linux-x64.zip
echo "7c201e1f16e64906da8fa7b77b3bda62e9c570f44323db763b4d083e0a1dbcd2  sonar-scanner-cli.zip" | sha256sum -c

unzip sonar-scanner-cli.zip -d $(dirname $SONAR_HOME)
mv -T $(dirname $SONAR_HOME)/sonar-scanner-* $SONAR_HOME
rm  -rf sonar-scanner-cli.zip
