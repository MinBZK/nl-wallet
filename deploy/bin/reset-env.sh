#!/usr/bin/env bash

set -euo pipefail

for name in $(helm list -q); do
    helm uninstall $name
done

REPLICAS=$(kubectl get deployment/static-files -o=jsonpath='{.spec.replicas}')
restore() {
    kubectl scale --replicas $REPLICAS deployment/static-files
}
trap restore EXIT

kubectl scale --replicas 0 deployment/static-files

kubectl delete pvc --wait=true \
    demo-issuer-issuance-server \
    pid-issuer \
    preloaded-gba-v-data \
    wallet-provider
