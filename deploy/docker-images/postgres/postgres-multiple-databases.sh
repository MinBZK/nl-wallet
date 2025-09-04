#!/usr/bin/env bash
# Based on: https://github.com/mrts/docker-postgresql-multiple-databases

set -euo pipefail

if [[ -z ${POSTGRES_MULTIPLE_DATABASES:-} ]]; then
    echo "No multiple databases request found"
    exit
fi

export PGUSER="${POSTGRES_USER}"
for db in $(echo $POSTGRES_MULTIPLE_DATABASES | tr ',' ' '); do
    echo "Creating database $db"
    createdb -O $POSTGRES_USER $db
done
