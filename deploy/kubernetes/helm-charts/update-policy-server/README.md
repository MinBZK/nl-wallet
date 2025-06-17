## Parameters

### Global parameters

| Name                   | Description                  | Value |
| ---------------------- | ---------------------------- | ----- |
| `global.imageRegistry` | Global Docker image registry | `""`  |

### Common parameters

| Name               | Description                                   | Value |
| ------------------ | --------------------------------------------- | ----- |
| `nameOverride`     | String to partially override chart's fullname | `""`  |
| `extraPodLabels`   | Labels to add to all deployed objects         | `{}`  |
| `imagePullSecrets` | Array of secret names for private registries  | `[]`  |

### Image parameters

| Name               | Description                        | Value          |
| ------------------ | ---------------------------------- | -------------- |
| `image.repository` | Repository for the container image | `""`           |
| `image.pullPolicy` | Image pull policy                  | `IfNotPresent` |
| `image.tag`        | Image tag                          | `""`           |

### Deployment parameters

| Name           | Description        | Value |
| -------------- | ------------------ | ----- |
| `replicaCount` | Number of replicas | `2`   |

### Service Account configuration

| Name                 | Description                 | Value |
| -------------------- | --------------------------- | ----- |
| `serviceAccountName` | Name of the service account | `""`  |

### Security context

| Name              | Description                        | Value |
| ----------------- | ---------------------------------- | ----- |
| `securityContext` | Security context for the container | `{}`  |

### Ingress parameters

| Name                    | Description                            | Value   |
| ----------------------- | -------------------------------------- | ------- |
| `ingress.className`     | Ingress class name                     | `nginx` |
| `ingress.hostname`      | Hostname for the ingress               | `nil`   |
| `ingress.contextPath`   | Optional context path for the ingress  | `nil`   |
| `ingress.tlsSecretName` | TLS secret name for the ingress        | `nil`   |
| `ingress.labels`        | Additional labels for the ingress      | `{}`    |
| `ingress.annotations`   | Additional annotations for the ingress | `{}`    |

### Resource requests and limits

| Name                        | Description    | Value  |
| --------------------------- | -------------- | ------ |
| `resources.requests.cpu`    | CPU request    | `50m`  |
| `resources.requests.memory` | Memory request | `32Mi` |
| `resources.limits.cpu`      | CPU limit      | `200m` |
| `resources.limits.memory`   | Memory limit   | `64Mi` |

### Volumes

| Name                        | Description                                | Value                                 |
| --------------------------- | ------------------------------------------ | ------------------------------------- |
| `volumes[0].name`           | Name of the first volume                   | `config-volume`                       |
| `volumes[0].configMap.name` | Name of the ConfigMap for the first volume | `nl-wallet-update-policy-server-data` |

### Volume mounts

| Name                        | Description                            | Value                        |
| --------------------------- | -------------------------------------- | ---------------------------- |
| `volumeMounts[0].name`      | Name of the first volume mount         | `config-volume`              |
| `volumeMounts[0].mountPath` | Mount path for the first volume mount  | `/update_policy_server.toml` |
| `volumeMounts[0].subPath`   | Sub-path within the first volume mount | `config.toml`                |

