#!/usr/bin/env bash
set -euxo pipefail

wget --no-hsts -q -O flutter.tar.xz https://storage.googleapis.com/flutter_infra_release/releases/stable/linux/flutter_linux_3.47.5-stable.tar.xz
echo "2132e990f236f8d22e7c6314b29a191a95b10d7cbcfec9b4e2e303d996652cbb  flutter.tar.xz" | sha256sum -c

tar -xf flutter.tar.xz
rm flutter.tar.xz

mv -T flutter $FLUTTER_HOME

# Needed to be able to run flutter as root
# (although this is run as user, root will have user's home at the end)
git config --global --add safe.directory $FLUTTER_HOME

dart --disable-analytics
flutter config --no-analytics

flutter precache --android
flutter doctor --android-licenses
flutter precache --ios

dart pub global activate junitreport 2.0.2
