## Parameters

### Global parameters

| Name                   | Description                  | Value |
| ---------------------- | ---------------------------- | ----- |
| `global.imageRegistry` | Global Docker image registry | `""`  |

### Common parameters

| Name           | Description                                   | Value |
| -------------- | --------------------------------------------- | ----- |
| `nameOverride` | String to partially override chart's fullname | `""`  |

### Common parameters

| Name           | Description        | Value |
| -------------- | ------------------ | ----- |
| `replicaCount` | Number of replicas | `2`   |

### Image parameters

| Name               | Description                        | Value          |
| ------------------ | ---------------------------------- | -------------- |
| `image.repository` | Repository for the container image | `nil`          |
| `image.pullPolicy` | Image pull policy                  | `IfNotPresent` |
| `image.tag`        | Image tag                          | `nil`          |

### Image migration parameters

| Name                         | Description                                   | Value          |
| ---------------------------- | --------------------------------------------- | -------------- |
| `imageMigrations.repository` | Repository for the migration container image  | `nil`          |
| `imageMigrations.pullPolicy` | Image pull policy for the migration container | `IfNotPresent` |

### Pod labels

| Name             | Description                    | Value |
| ---------------- | ------------------------------ | ----- |
| `extraPodlabels` | Additional labels for the pods | `{}`  |

### Image pull secrets

| Name               | Description                                  | Value |
| ------------------ | -------------------------------------------- | ----- |
| `imagePullSecrets` | Array of secret names for private registries | `[]`  |

### Ingress internal parameters

| Name                            | Description                                     | Value   |
| ------------------------------- | ----------------------------------------------- | ------- |
| `ingressInternal.enabled`       | Enable or disable the internal ingress          | `false` |
| `ingressInternal.className`     | Ingress class name                              | `nginx` |
| `ingressInternal.hostname`      | Hostname for the internal ingress               | `nil`   |
| `ingressInternal.tlsSecretName` | Name of the TLS secret for the internal ingress | `nil`   |
| `ingressInternal.labels`        | Additional labels for the internal ingress      | `{}`    |
| `ingressInternal.annotations`   | Additional annotations for the internal ingress | `{}`    |

### Ingress parameters

| Name                    | Description                            | Value   |
| ----------------------- | -------------------------------------- | ------- |
| `ingress.enabled`       | Enable or disable the ingress          | `true`  |
| `ingress.className`     | Ingress class name                     | `nginx` |
| `ingress.hostname`      | Hostname for the ingress               | `nil`   |
| `ingress.contextPath`   | Optional context path for the ingress  | `nil`   |
| `ingress.tlsSecretName` | Name of the TLS secret for the ingress | `nil`   |
| `ingress.labels`        | Additional labels for the ingress      | `{}`    |
| `ingress.annotations`   | Additional annotations for the ingress | `{}`    |

### Resource requests and limits

| Name                        | Description                                                       | Value   |
| --------------------------- | ----------------------------------------------------------------- | ------- |
| `resources.requests.cpu`    | CPU request                                                       | `50m`   |
| `resources.requests.memory` | Memory request                                                    | `64Mi`  |
| `resources.limits.cpu`      | CPU limit                                                         | `200m`  |
| `resources.limits.memory`   | Memory limit                                                      | `128Mi` |
| `database.secretName`       | The secret name that contains the connection url for the database | `nil`   |

### Urls

| Name                   | Description                  | Value |
| ---------------------- | ---------------------------- | ----- |
| `universalLinkBaseUrl` | Base URL for universal links | `nil` |

### Migration parameters

| Name               | Description                             | Value  |
| ------------------ | --------------------------------------- | ------ |
| `migration.labels` | Additional labels for the migration job | `{}`   |
| `migration.reset`  | Enable reset cron job                   | `true` |

