set -euxo pipefail

# Get channel and version name from the branch/tag name
# i.e. release/0.1.1 or beta/0.4.2-fix-foo
release_channel="$(echo $GITHUB_REF_NAME | cut -d/ -f 1)"
release_version="$(echo $GITHUB_REF_NAME | cut -d/ -f 2)"

# Use the Github CI run number as build number
release_number="$(( $GITHUB_RUN_NUMBER + 0 ))"

# Determine package name for the specified channel
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

# Determine APK name (without extension) and full path
apk_name="${package_name}_${release_number}"
apk_path="build/app/outputs/apk/release/${apk_name}.apk"

# Put name in env for upload artifact step
echo "APK_NAME=$apk_name" >> $GITHUB_ENV