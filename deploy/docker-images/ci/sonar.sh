#!/usr/bin/env bash
set -euxo pipefail

wget --no-hsts -q -O sonar-scanner-cli.zip https://binaries.sonarsource.com/Distribution/sonar-scanner-cli/sonar-scanner-cli-6.2.1.4610-linux-x64.zip
echo "0b8a3049f0bd5de7abc1582c78c233960d3d4ed7cc983a1d1635e8552f8bb439  sonar-scanner-cli.zip" | sha256sum -c

unzip sonar-scanner-cli.zip -d $(dirname $SONAR_HOME)
mv -T $(dirname $SONAR_HOME)/sonar-scanner-* $SONAR_HOME
rm  -rf sonar-scanner-cli.zip
