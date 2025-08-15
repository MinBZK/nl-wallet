#!/usr/bin/env bash
set -euxo pipefail

wget --no-hsts -q -O sonar-scanner-cli.zip https://binaries.sonarsource.com/Distribution/sonar-scanner-cli/sonar-scanner-cli-7.2.0.5079-linux-x64.zip
echo "da9f4e64a3d555f08ce38b5469ebd91fe2b311af473f7001a5ee5c1fd58b004b  sonar-scanner-cli.zip" | sha256sum -c

unzip sonar-scanner-cli.zip -d $(dirname $SONAR_HOME)
mv -T $(dirname $SONAR_HOME)/sonar-scanner-* $SONAR_HOME
rm  -rf sonar-scanner-cli.zip
