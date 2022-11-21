#!/usr/bin/env bash
set -euxo pipefail

. ./.github/workflows/calculate_metadata.sh

flutter config --no-analytics
flutter doctor
flutter pub get

# Rename package and build, with replaced build number and build version
flutter pub run change_app_package_name:main "$package_name"
flutter build apk \
  --build-number "$release_number" \
  --build-name "$release_version" \
  --dart-define "commit_sha=$GITHUB_SHA"

# Rename APK to package name and build number
mv "build/app/outputs/apk/release/app-release.apk" "$apk_path"
