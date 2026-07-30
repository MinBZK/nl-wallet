ARG HARBOR_REGISTRY

FROM ${HARBOR_REGISTRY}/docker-hub-proxy/nginxinc/nginx-unprivileged:alpine

COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY dist/ /usr/share/nginx/html/

EXPOSE 8080
