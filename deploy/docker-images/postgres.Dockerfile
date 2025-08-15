# Based on: https://github.com/mrts/docker-postgresql-multiple-databases
ARG DOCKER_HUB_PROXY
# 17.5.0-debian-12-r20
FROM ${DOCKER_HUB_PROXY}bitnami/postgresql@sha256:42a8200d35971f931b869ef5252d996e137c6beb4b8f1b6d2181dc7d1b6f62e0
COPY postgres-multiple-databases.sh /docker-entrypoint-initdb.d/
