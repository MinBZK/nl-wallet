# Keycloak helm-chart

Deploys [Keycloak](https://www.keycloak.org/) as the OIDC provider for the NL
Wallet revocation portal. Imports the `nl-wallet` realm (roles, seed users, the
`wallet-backend` client) on first start. For ont/demo; for local development use
`scripts/start-devenv.sh keycloak` (see
`wallet_docs/development/keycloak-setup.md`).

## Prerequisites

- Kubernetes with the Gateway API (a gateway terminates TLS; the pod serves
  plain HTTP on `:8080`).
- Helm.
- Two secrets in the namespace:
  - `nl-wallet-keycloak-admin` (`keycloak.admin.existingSecret`): keys
    `username` / `password` for the bootstrap admin.
  - `nl-wallet-keycloak-users` (`keycloak.usersSecret`): one key per seed user
    (`administrator-password`, `manager-password`, `destroyer-password`).

## Installing

```shell
# Resolve the sp-common dependency (required before a fresh install):
helm dependency build deploy/helm-charts/keycloak

# Override the values that have no usable default:
helm install keycloak deploy/helm-charts/keycloak \
  --set keycloak.hostname=keycloak.example.org \
  --set keycloak.clientRedirectUri=https://wallet.example.org/auth/callback \
  --set 'httpRoute.parentRefs[0].name=<gateway>' \
  --set 'httpRoute.hostnames[0]=keycloak.example.org'
```

Seed-user passwords stay out of the chart: `realm.json` uses
`${KC_PASSWORD_FOR_<USERNAME>}` placeholders that Keycloak substitutes at import
time from env vars sourced (via `secretKeyRef`) from `keycloak.usersSecret`.

Two caveats (detail in `wallet_docs/development/keycloak-setup.md`):

- **Verify substitution.** If Keycloak does not replace the placeholders, the
  literal string becomes the password, and it is public. Confirm login with the
  literal `${KC_PASSWORD_FOR_ADMINISTRATOR}` is rejected.
- **Import runs once.** `--import-realm` only seeds a realm that does not exist;
  with `persistence.enabled` the realm stays on the PVC, so later user or
  password changes need the realm or PVC wiped.

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

| Name                            | Description                                  | Value   |
| ------------------------------- | -------------------------------------------- | ------- |
| `probes.config.liveness`        | Additional configuration for liveness probe  | `{}`    |
| `probes.config.readiness`       | Additional configuration for readiness probe | `{}`    |
| `probes.config.startup`         | Additional configuration for startup probe   | `{}`    |
| `probes.disableLiveness`        | Disable liveness probe                       | `false` |
| `probes.useLivenessAsReadiness` | Use liveness endpoint for readiness          | `false` |

### Keycloak parameters

| Name                                        | Description                                                                                                                                                                                                                                                                                           | Value                      |
| ------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------- |
| `keycloak.hostname`                         | Value for KC_HOSTNAME (external hostname of the ingress)                                                                                                                                                                                                                                              | `localhost`                |
| `keycloak.database`                         | Value for KC_DB (defaults to dev-file backed by the PVC)                                                                                                                                                                                                                                              | `dev-file`                 |
| `keycloak.proxyHeaders`                     | Value for KC_PROXY_HEADERS (xforwarded or forwarded)                                                                                                                                                                                                                                                  | `xforwarded`               |
| `keycloak.clientRedirectUri`                | The wallet-backend client redirect URI, rendered into realm.json                                                                                                                                                                                                                                      | `nil`                      |
| `keycloak.admin.existingSecret.name`        | Name of the secret holding the admin credentials                                                                                                                                                                                                                                                      | `nl-wallet-keycloak-admin` |
| `keycloak.admin.existingSecret.usernameKey` | Key in the secret for the admin username (KC_BOOTSTRAP_ADMIN_USERNAME)                                                                                                                                                                                                                                | `username`                 |
| `keycloak.admin.existingSecret.passwordKey` | Key in the secret for the admin password (KC_BOOTSTRAP_ADMIN_PASSWORD)                                                                                                                                                                                                                                | `password`                 |
| `keycloak.extraEnv`                         | Additional environment variables                                                                                                                                                                                                                                                                      | `[]`                       |
| `keycloak.extraImportFiles`                 | Additional/override files placed in the import directory (filename -> content), rendered as templates                                                                                                                                                                                                 | `{}`                       |
| `keycloak.usersSecret.name`                 | Name of the secret holding the seed user passwords (referenced by users[].passwordKey)                                                                                                                                                                                                                | `nl-wallet-keycloak-users` |
| `keycloak.users`                            | Seed users rendered into realm.json, each with username, passwordKey (key in usersSecret) and privileges (role names without the privilege_ prefix). The password is injected as env var KC_PASSWORD_FOR_<USERNAME> and substituted at import time, so usernames must be env-var-safe ([A-Za-z0-9_]). |                            |

### Persistence parameters

| Name                           | Description                         | Value           |
| ------------------------------ | ----------------------------------- | --------------- |
| `persistence.enabled`          | Enable a PVC for /opt/keycloak/data | `true`          |
| `persistence.accessMode`       | PVC access mode                     | `ReadWriteOnce` |
| `persistence.size`             | PVC size                            | `1Gi`           |
| `persistence.storageClassName` | Optional storage class              | `nil`           |
