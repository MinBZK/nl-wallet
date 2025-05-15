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

### Ingress parameters

| Name                    | Description                            | Value   |
| ----------------------- | -------------------------------------- | ------- |
| `ingress.enabled`       | Enable or disable the ingress          | `true`  |
| `ingress.className`     | Ingress class name                     | `nginx` |
| `ingress.tlsSecretName` | Name of the TLS secret for the ingress | `nil`   |
| `ingress.labels`        | Additional labels for the ingress      | `{}`    |

### Resource requests and limits

| Name                        | Description    | Value   |
| --------------------------- | -------------- | ------- |
| `resources.limits.memory`   | Memory limit   | `512Mi` |
| `resources.limits.cpu`      | CPU limit      | `500m`  |
| `resources.requests.memory` | Memory request | `512Mi` |
| `resources.requests.cpu`    | CPU request    | `300m`  |

### Issuance Server parameters

| Name                         | Description                          | Value |
| ---------------------------- | ------------------------------------ | ----- |
| `issuanceServer.hostname`    | Hostname for the issuance server     | `nil` |
| `issuanceServer.contextPath` | Context path for the issuance server | `nil` |
| `universalLinkBaseUrl`       | Base URL for universal links         | `nil` |

### Migration parameters

| Name               | Description                             | Value  |
| ------------------ | --------------------------------------- | ------ |
| `migration.labels` | Additional labels for the migration job | `{}`   |
| `migration.reset`  | Enable reset cron job                   | `true` |
