ARG DOCKER_HUB_PROXY
# 17.6-trixie
FROM ${DOCKER_HUB_PROXY}library/postgres@sha256:29e0bb09c8e7e7fc265ea9f4367de9622e55bae6b0b97e7cce740c2d63c2ebc0
COPY postgres-multiple-databases.sh /docker-entrypoint-initdb.d/
