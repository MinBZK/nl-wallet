#!/usr/bin/env sh

set -eu

sed -i -E "s#(data-environment=\")(\")#\1$ADMIN_PORTAL_ENVIRONMENT\2#" /usr/share/nginx/html/index.html
