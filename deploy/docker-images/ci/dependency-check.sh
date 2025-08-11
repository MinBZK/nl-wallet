#!/usr/bin/env bash
set -euxo pipefail

VERSION=12.1.3

gpg --keyserver hkp://keyserver.ubuntu.com --recv-keys 259A55407DD6C00299E6607EFFDE55BE73A2D1ED
wget --no-hsts -q https://github.com/dependency-check/DependencyCheck/releases/download/v${VERSION}/dependency-check-${VERSION}-release.zip
wget --no-hsts -q https://github.com/dependency-check/DependencyCheck/releases/download/v${VERSION}/dependency-check-${VERSION}-release.zip.asc
gpg --verify dependency-check-${VERSION}-release.zip.asc

unzip dependency-check-${VERSION}-release.zip -d $(dirname $DEPENDENCY_CHECK_HOME)
rm dependency-check-${VERSION}-release.zip dependency-check-${VERSION}-release.zip.asc

rm -rf ~/.gnupg
