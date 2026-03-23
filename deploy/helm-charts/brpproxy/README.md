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
| `replicaCount` | Number of replicas | `1`   |

### Image parameters

| Name               | Description                        | Value                                                    |
| ------------------ | ---------------------------------- | -------------------------------------------------------- |
| `image.repository` | Repository for the container image | `ghcr-io-proxy/brp-api/haal-centraal-brp-bevragen-proxy` |
| `image.pullPolicy` | Image pull policy                  | `IfNotPresent`                                           |
| `image.tag`        | Image tag                          | `2.1.2`                                                  |

### Image pull secrets

| Name               | Description                                  | Value |
| ------------------ | -------------------------------------------- | ----- |
| `imagePullSecrets` | Array of secret names for private registries | `[]`  |

### Deployment parameters

| Name                                 | Description                                                            | Value |
| ------------------------------------ | ---------------------------------------------------------------------- | ----- |
| `deployment.strategy`                | Strategy used to replace old pods by new one                           | `nil` |
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
| `resources.requests.cpu`    | CPU request    | `100m`  |
| `resources.requests.memory` | Memory request | `128Mi` |
| `resources.limits.cpu`      | CPU limit      | `500m`  |
| `resources.limits.memory`   | Memory limit   | `256Mi` |

### Routes configuration

| Name                            | Description                  | Value              |
| ------------------------------- | ---------------------------- | ------------------ |
| `routes[0].scheme`              | URL scheme (http/https)      | `http`             |
| `routes[0].downstreams[0].host` | Downstream host of the route | `gba-hc-converter` |
| `routes[0].downstreams[0].port` | Downstream port of the route | `80`               |
