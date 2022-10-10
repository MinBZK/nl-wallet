#!/usr/bin/env bash
set -euxo pipefail

flutter config --no-analytics
flutter doctor
flutter pub get

# Get channel and version name from the branch/tag name
# i.e. release/0.1.1 or beta/0.4.2-fix-foo
release_channel="$(echo $GITHUB_REF_NAME | cut -d/ -f 1)"
release_version="$(echo $GITHUB_REF_NAME | cut -d/ -f 2)"

# Rename package name to the specified channel
case "$release_channel" in
  release)
    package_name="$ANDROID_PACKAGE_NAME"
    ;;

  alpha)
    package_name="$ANDROID_PACKAGE_NAME.alpha"
    ;;

  beta)
    package_name="$ANDROID_PACKAGE_NAME.beta"
    ;;

  *)
    echo "Invalid release channel '$release_channel'"
    exit 1
    ;;
esac

# Rename package and build, with replaced build number and build version
flutter pub run change_app_package_name:main "$package_name"
flutter build apk \
  --build-number "$(( $GITHUB_RUN_NUMBER + 0 ))" \
  --build-name "$release_version" \
  --dart-define "commit_sha=$GITHUB_SHA"

# Rename APK to package name and build number, put in environment
apk_name="${package_name}_${GITHUB_RUN_NUMBER}"
apk_path="build/app/outputs/apk/release/${apk_name}.apk"
mv "build/app/outputs/apk/release/app-release.apk" "$apk_path"

echo "APK_NAME=$apk_name" >> $GITHUB_ENV
echo "APK_PATH=$apk_path" >> $GITHUB_ENV
