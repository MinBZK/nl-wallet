#!/bin/zsh

set -euo pipefail

eval "$(/opt/homebrew/bin/brew shellenv)"

cd -- "$( dirname -- ${(%):-%N} )"
packer init .
packer build wallet.pkr.hcl
