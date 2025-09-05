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
| `resources.requests.memory` | Memory request | `16Mi` |
| `resources.limits.cpu`      | CPU limit      | `200m` |
| `resources.limits.memory`   | Memory limit   | `32Mi` |

### Update policy

| Name                         | Description                                            | Value                                 |
| ---------------------------- | ------------------------------------------------------ | ------------------------------------- |
| `updatePolicy.configMapName` | Name to the the ConfigMap containing the update policy | `nl-wallet-update-policy-server-data` |
| `updatePolicy.configMapKey`  | Key of the the ConfigMap containing the update policy  | `config.toml`                         |

