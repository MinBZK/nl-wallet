ARG DOCKER_HUB_PROXY
# 18.2-trixie
FROM ${DOCKER_HUB_PROXY}library/postgres@sha256:b6b4d0b75c699a2c94dfc5a94fe09f38630f3b67ab0e1653ede1b7ac8e13c197
COPY postgres-multiple-databases.sh /docker-entrypoint-initdb.d/
