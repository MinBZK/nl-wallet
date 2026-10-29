#!/usr/bin/env bash
set -euxo pipefail

wget --no-hsts -q -O sonar-scanner-cli.zip https://binaries.sonarsource.com/Distribution/sonar-scanner-cli/sonar-scanner-cli-8.1.0.6389-linux-x64.zip
echo "bb8f709f9cb73352f8d1260a3b3c506c0f41146754bc630762c126d795499d0b  sonar-scanner-cli.zip" | sha256sum -c

base="$(dirname "$SONAR_HOME")"
unzip sonar-scanner-cli.zip -d "$base"
mv -T "$base"/sonar-scanner-* "$SONAR_HOME"
rm  -rf sonar-scanner-cli.zip
