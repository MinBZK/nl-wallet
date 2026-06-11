#!/usr/bin/env bash

set -euo pipefail

# Uninstall all helm installs
for name in $(helm list -q); do
    helm uninstall $name
done

# Clean WIA status lists on static server
POD=$(kubectl get pod -l 'app.kubernetes.io/name=static-files' -o=name | head -n 1)
kubectl exec $POD -- sh -c 'rm -rf /usr/share/nginx/html/wia/*'

# Delete PVCs created by helm install
kubectl delete pvc --wait=true \
    demo-issuer-issuance-server \
    demo-issuer-pacf-issuance-server \
    pid-issuer \
    preloaded-gba-v-data \
    data-redis-0
