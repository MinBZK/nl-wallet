#!/usr/bin/env bash
set -euxo pipefail

wget --no-hsts -q -O flutter.tar.xz https://storage.googleapis.com/flutter_infra_release/releases/stable/linux/flutter_linux_3.35.1-stable.tar.xz
echo "58efd9d1e570a1bf976e218cfbbcca3f23b21b873d765a74e045d5b9022ab515  flutter.tar.xz" | sha256sum -c

tar -xf flutter.tar.xz
rm flutter.tar.xz

mv -T flutter $FLUTTER_HOME

# Needed to be able to run flutter as root
# (although this is run as user, root will have user's home at the end)
git config --global --add safe.directory $FLUTTER_HOME

dart --disable-analytics
flutter config --no-analytics

flutter precache
flutter doctor --android-licenses

dart pub global activate junitreport 2.0.2
