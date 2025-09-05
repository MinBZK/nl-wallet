#!/usr/bin/env bash

set -euo pipefail

if ! command -v cmake > /dev/null; then
    echo "ERROR: cmake is required to build softhsm from source"
    exit 1
fi


CURPWD="$(PWD)"
WORKDIR="$(mktemp -d)"
echo "Working in $WORKDIR"
echo "(only cleaned up when script succeeds)"
echo


INSTALL_DIR="${1-/opt/softhsm}"
echo "Installing into $INSTALL_DIR"

if [[ ! -d $INSTALL_DIR ]]; then
    echo "Creating install dir"
    sudo mkdir "$INSTALL_DIR"
    if uname -a | grep -i darwin >/dev/null; then
        sudo chown "${USER}:admin" "$INSTALL_DIR"
    fi
fi
echo


echo "Downloading ..."
cd $WORKDIR
curl -fsSL 'https://github.com/softhsm/SoftHSMv2/archive/refs/heads/develop.tar.gz' | tar -xz
cd SoftHSMv2-*

echo "Building ..."
cmake -S . -B build \
    -DCMAKE_INSTALL_PREFIX="$INSTALL_DIR" \
    -DCMAKE_INSTALL_SYSCONFDIR="$INSTALL_DIR/etc" \
    -DCMAKE_INSTALL_LOCALSTATEDIR="$INSTALL_DIR/var" \
    -DENABLE_STATIC=OFF
cmake --build build

echo "Installing ..."
cmake --install build


echo "Cleaning up"
cd "$CURPWD"
rm -rf "$WORKDIR"

echo "DONE"
echo
echo "Add '$INSTALL_DIR/bin' to your PATH"
