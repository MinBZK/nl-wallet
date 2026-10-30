#!/usr/bin/env bash
set -euxo pipefail

pipx install --include-deps ansible==14.5.0

# Inject to be installed in same venv
pipx inject ansible ansible-lint==26.10.1
ln -s ~/.local/share/pipx/venvs/ansible/bin/ansible-lint .local/bin

rm -rf ~/.cache
