#!/usr/bin/env bash

set -euo pipefail

# Uninstall all helm installs
for name in $(helm list -q); do
    helm uninstall $name
done

# Clean WUA status lists on static server
POD=$(kubectl get pod -l app=static-files -o=name | head -n 1)
kubectl exec -t $POD -- sh -c 'rm -rf /usr/share/nginx/html/wua/*'

# Delete PVCs created by helm install
kubectl delete pvc --wait=true \
    demo-issuer-issuance-server \
    pid-issuer \
    preloaded-gba-v-data
