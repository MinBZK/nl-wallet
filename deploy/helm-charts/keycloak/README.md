# Keycloak helm-chart

Deploys [Keycloak](https://www.keycloak.org/) as the OIDC provider for the NL
Wallet revocation portal. Imports the `nl-wallet` realm (roles, users, client)
on first start. (see also: `wallet_docs/development/keycloak-setup.md`).

## Prerequisites

- Kubernetes with a deployed gateway
- Helm + this helm-chart.
- If your environment requires labels to be set so you can access the pod (for
  example, if your environment utilizes NetworkPolicy objects), then make sure
  you add those to `extraPodLabels`.
- Two secrets in the namespace:

  ```bash
  # Bootstrap admin credentials (KC_BOOTSTRAP_ADMIN_USERNAME/PASSWORD):
  kubectl create secret generic nl-wallet-keycloak-admin \
    --from-literal=username=keycloak --from-literal=password=<some-password>

  # To-be-imported realm.json:
  kubectl create secret generic nl-wallet-keycloak-realm \
    --from-file=realm.json=scripts/devenv/keycloak/import/realm.json
  ```

  The realm JSON holds roles, users, passwords, and client definitions (which
  have `redirectUris` configured).

## Installing

This example targets the `nl-wallet-ont` and `nl-wallet-demo` namespaces on
the `test-a` and `test-b` clusters; adjust for other environments. Assumes
the prerequisites above (admin + realm secrets) are already in place.

Step-by-step:

1. Build the chart dependency (required once after a fresh checkout or after
   bumping `sp-common`):

   ```bash
   helm dependency build deploy/helm-charts/keycloak
   ```

2. Create a values override file (e.g. `values-ont.yaml`) for the settings
   that have no usable default. For ont/demo this typically routes the image
   pull through the Harbor proxy, points the HTTPRoute at a "private" gateway
   with its wildcard cert, and admits the pod to the gateway via the
   a label that is associated with the correct network policy. Here is an
   example that matches our setup somewhat:

   ```yaml
   global:
     imageRegistry: harbor.example.com
   image:
     repository: quay-proxy/keycloak/keycloak
   persistence:
     enabled: true
   httpRoute:
     enabled: true
     parentRefs:
       - name: gateway-private
     hostnames:
       - keycloak.nl-wallet-ont.example.com
   keycloak:
     hostname: keycloak.nl-wallet-ont.example.com
   extraPodLabels:
     ingress-controller-frontoffice-policy: allow
   ```

3. Install or upgrade:

   ```bash
   helm upgrade --install keycloak deploy/helm-charts/keycloak \
     --namespace nl-wallet-ont -f values-ont.yaml
   kubectl rollout status deployment/keycloak -n nl-wallet-ont
   ```

   A cold start reaches `Ready` in ~60s to 80s on our environment; the chart's
   startup probe defaults allow for this leisurely pace.

4. Log in at `https://<hostname>/` with the credentials configured in
   `nl-wallet-keycloak-admin`. With a `nl-wallet-keycloak-realm` present, you'll
   see the `nl-wallet` realm in addition to `master` realm.

   Note that import runs once. `--import-realm` only imports a realm that does
   not exist; with `persistence.enabled` the realm stays on the PVC, so later
   realm changes in a realm secret are not applied until you remove the PVC.
   To re-import, wipe the realm or PVC.

## Parameters

### Global parameters

| Name                   | Description                  | Value |
| ---------------------- | ---------------------------- | ----- |
| `global.imageRegistry` | Global Docker image registry | `""`  |

### Common parameters

| Name               | Description                                    | Value |
| ------------------ | ---------------------------------------------- | ----- |
| `fullnameOverride` | String to completely override chart's fullname | `""`  |
| `nameOverride`     | String to partially override chart's fullname  | `""`  |
| `replicaCount`     | Number of replicas                             | `1`   |

### Image parameters

| Name               | Description                              | Value                       |
| ------------------ | ---------------------------------------- | --------------------------- |
| `image.repository` | Repository for the container image       | `quay.io/keycloak/keycloak` |
| `image.pullPolicy` | Image pull policy                        | `IfNotPresent`              |
| `image.tag`        | Image tag (defaults to Chart.appVersion) | `nil`                       |

### Image pull secrets

| Name               | Description                                  | Value |
| ------------------ | -------------------------------------------- | ----- |
| `imagePullSecrets` | Array of secret names for private registries | `[]`  |

### Deployment parameters

| Name                                 | Description                                                            | Value |
| ------------------------------------ | ---------------------------------------------------------------------- | ----- |
| `deployment.strategy`                | Strategy used to replace old pods by new ones                          | `nil` |
| `deployment.revisionHistoryLimit`    | The number of old ReplicaSets to retain to allow rollback              | `2`   |
| `deployment.progressDeadlineSeconds` | The number of seconds you want to wait for your Deployment to progress | `300` |

### Security parameters

| Name                 | Description                        | Value |
| -------------------- | ---------------------------------- | ----- |
| `serviceAccountName` | Name of the service account        | `nil` |
| `podSecurityContext` | Security context for the pod       | `{}`  |
| `securityContext`    | Security context for the container | `{}`  |

### Annotations and labels

| Name               | Description                               | Value |
| ------------------ | ----------------------------------------- | ----- |
| `extraAnnotations` | Additional annotations for the deployment | `{}`  |
| `extraPodLabels`   | Additional labels for the pods            | `{}`  |

### Resource requests and limits

| Name                        | Description    | Value   |
| --------------------------- | -------------- | ------- |
| `resources.requests.cpu`    | CPU request    | `200m`  |
| `resources.requests.memory` | Memory request | `512Mi` |
| `resources.limits.cpu`      | CPU limit      | `1000m` |
| `resources.limits.memory`   | Memory limit   | `1Gi`   |

### HTTP route parameters

| Name                    | Description                          | Value  |
| ----------------------- | ------------------------------------ | ------ |
| `httpRoute.enabled`     | Enable or disable the route          | `true` |
| `httpRoute.parentRefs`  | Parent references to the gateway     | `[]`   |
| `httpRoute.hostnames`   | Hostnames for the route              | `[]`   |
| `httpRoute.contextPath` | Optional context path for the route  | `nil`  |
| `httpRoute.labels`      | Additional labels for the route      | `{}`   |
| `httpRoute.annotations` | Additional annotations for the route | `{}`   |

### Container probes

| Name                                     | Description                                                              | Value    |
| ---------------------------------------- | ------------------------------------------------------------------------ | -------- |
| `probes.config.liveness`                 | Additional configuration for liveness probe                              | `{}`     |
| `probes.config.readiness`                | Additional configuration for readiness probe                             | `{}`     |
| `probes.config.startup.periodSeconds`    | Seconds between startup probe checks                                     | `10`     |
| `probes.config.startup.failureThreshold` | Number of failed startup probes before the container is restarted        | `15`     |
| `probes.port`                            | Named container port for probe targets (defaults to "http" in sp-common) | `health` |
| `probes.disableLiveness`                 | Disable liveness probe                                                   | `false`  |
| `probes.useLivenessAsReadiness`          | Use liveness endpoint for readiness                                      | `false`  |

### Keycloak parameters

| Name                                        | Description                                                                                       | Value                      |
| ------------------------------------------- | ------------------------------------------------------------------------------------------------- | -------------------------- |
| `keycloak.database`                         | Value for KC_DB (defaults to dev-file backed by the PVC)                                          | `dev-file`                 |
| `keycloak.proxyHeaders`                     | Value for KC_PROXY_HEADERS (xforwarded or forwarded)                                              | `xforwarded`               |
| `keycloak.realm.existingSecret.name`        | Name of the secret holding the realm import JSON (the nl-wallet realm; must exist before install) | `nl-wallet-keycloak-realm` |
| `keycloak.realm.existingSecret.key`         | Key in the secret whose value is the realm JSON file                                              | `realm.json`               |
| `keycloak.admin.existingSecret.name`        | Name of the secret holding the admin credentials                                                  | `nl-wallet-keycloak-admin` |
| `keycloak.admin.existingSecret.usernameKey` | Key in the secret for the admin username (KC_BOOTSTRAP_ADMIN_USERNAME)                            | `username`                 |
| `keycloak.admin.existingSecret.passwordKey` | Key in the secret for the admin password (KC_BOOTSTRAP_ADMIN_PASSWORD)                            | `password`                 |
| `keycloak.extraEnv`                         | Additional environment variables                                                                  | `[]`                       |

### Persistence parameters

| Name                           | Description                         | Value           |
| ------------------------------ | ----------------------------------- | --------------- |
| `persistence.enabled`          | Enable a PVC for /opt/keycloak/data | `true`          |
| `persistence.accessMode`       | PVC access mode                     | `ReadWriteOnce` |
| `persistence.size`             | PVC size                            | `1Gi`           |
| `persistence.storageClassName` | Optional storage class              | `nil`           |
