#!/usr/bin/env bash
set -euxo pipefail

wget --no-hsts -q -O flutter.tar.xz https://storage.googleapis.com/flutter_infra_release/releases/stable/linux/flutter_linux_3.32.1-stable.tar.xz
echo "ee8701ceee151830480ec1692cf510a4101186caf6451516f78827dddb3e460f  flutter.tar.xz" | sha256sum -c

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
