#!/usr/bin/env bash
set -euxo pipefail

wget --no-hsts -q -O flutter.tar.xz https://storage.googleapis.com/flutter_infra_release/releases/stable/linux/flutter_linux_3.41.2-stable.tar.xz
echo "4a04f8a6152986d14fc137ffaf98106ca743c0f9ab66f1bc2f20ee84eb573e5c  flutter.tar.xz" | sha256sum -c

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

dart pub global activate junitreport 2.0.2
