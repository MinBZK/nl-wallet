#!/usr/bin/env bash

set -eu

source "$(cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd)/store-functions.sh"

check

target=${@:$#}
for source in ${@:1:$#-1}; do
    store $source $target
done
