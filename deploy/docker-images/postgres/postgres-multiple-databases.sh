#!/bin/bash
set -euo pipefail

if [[ -z ${POSTGRESQL_MULTIPLE_DATABASES:-} ]]; then
   echo "No multiple databases request found"
   exit
fi

for db in $(echo $POSTGRESQL_MULTIPLE_DATABASES | tr ',' ' '); do
   echo "Creating database $db"
    PGPASSWORD="${POSTGRESQL_INITSCRIPTS_PASSWORD}" \
       createdb -U $POSTGRESQL_INITSCRIPTS_USERNAME \
          -O $POSTGRESQL_USERNAME $db
done
