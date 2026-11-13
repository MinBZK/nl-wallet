ARG HARBOR_REGISTRY

FROM ${HARBOR_REGISTRY}/docker-hub-proxy/nginx/nginx-unprivileged:1.31.8-alpine3.24@sha256:26277f4923331a5ecda9096c0e378de65a30a5037129a8036c3ef5d9feee833d

USER root
# The [10-listen-on-ipv6-by-default.sh]
# https://github.com/nginx/docker-nginx-unprivileged/blob/main/entrypoint/10-listen-on-ipv6-by-default.sh
# script in this image adds listen to ipv6 interface if available, but only if
# the default.conf is unmodified with respect to the packaged file. The check
# is implemented with `apk manifest`, which sometimes tries to download the
# APKINDEX file. This unwanted behaviour. The whole check will fail anyway
# because we supply our own nginx config.
RUN rm /docker-entrypoint.d/10-listen-on-ipv6-by-default.sh \
 && chown 101:101 /usr/share/nginx/html
ADD docker/*.sh /docker-entrypoint.d/
USER 101

COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --chown=101 dist/ /usr/share/nginx/html/

EXPOSE 8080
