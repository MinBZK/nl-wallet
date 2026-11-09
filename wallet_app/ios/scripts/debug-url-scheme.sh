#!/bin/sh
#
# Any app can claim a custom URL scheme, so the debug scheme is only registered
# in builds without universal links (e.g. mock), like ulIntentFilter in
# android/app/build.gradle.kts. It is left out of the source Info.plist so that
# a build which skips this script never exposes it.
#
# Runs as an Xcode build phase, before code signing. It also removes the scheme,
# because an incremental build can reuse the built Info.plist of an earlier
# build without universal links.

set -e

DEBUG_SCHEME="walletdebuginteraction"
INFO_PLIST="${TARGET_BUILD_DIR}/${INFOPLIST_PATH}"
SCHEMES_KEY=":CFBundleURLTypes:0:CFBundleURLSchemes"

# UL_HOSTNAME can come from the environment, which also drives the applinks
# entitlement, or from a dart-define. Either one means universal links, and an
# empty dart-define must not override a hostname from the environment.
# Flutter passes dart-defines to Xcode as a comma-separated list of
# base64-encoded KEY=VALUE pairs.
UL_HOSTNAME="${UL_HOSTNAME:-}"
IFS=","
for DART_DEFINE in $DART_DEFINES; do
    DECODED=$(echo "${DART_DEFINE}" | base64 -d)
    if [ "${DECODED%%=*}" = "UL_HOSTNAME" ] && [ -n "${DECODED#*=}" ]; then
        UL_HOSTNAME="${DECODED#*=}"
    fi
done
unset IFS

debug_scheme_index() {
    INDEX=0
    while SCHEME=$(/usr/libexec/PlistBuddy -c "Print ${SCHEMES_KEY}:${INDEX}" "$INFO_PLIST" 2>/dev/null); do
        if [ "$SCHEME" = "$DEBUG_SCHEME" ]; then
            echo "$INDEX"
            return
        fi
        INDEX=$((INDEX + 1))
    done
}

INDEX=$(debug_scheme_index)

if [ -z "$UL_HOSTNAME" ] && [ -z "$INDEX" ]; then
    /usr/libexec/PlistBuddy -c "Add ${SCHEMES_KEY}:0 string ${DEBUG_SCHEME}" "$INFO_PLIST"
    echo "Added ${DEBUG_SCHEME} URL scheme"
elif [ -n "$UL_HOSTNAME" ] && [ -n "$INDEX" ]; then
    /usr/libexec/PlistBuddy -c "Delete ${SCHEMES_KEY}:${INDEX}" "$INFO_PLIST"
    echo "Removed ${DEBUG_SCHEME} URL scheme for UL hostname ${UL_HOSTNAME}"
fi
