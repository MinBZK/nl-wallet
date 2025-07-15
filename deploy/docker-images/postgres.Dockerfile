# Based on: https://github.com/mrts/docker-postgresql-multiple-databases
ARG DOCKER_HUB_PROXY
# 17.5.0-debian-12-r18
FROM ${DOCKER_HUB_PROXY}bitnami/postgresql@sha256:68bc11736c11e5a90675a0c25e78b9f2b82774d44d74996464adad6d12de2afa
COPY postgres-multiple-databases.sh /docker-entrypoint-initdb.d/
