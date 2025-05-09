## Parameters

### Global parameters

| Name                   | Description                  | Value |
| ---------------------- | ---------------------------- | ----- |
| `global.imageRegistry` | Global Docker image registry | `""`  |

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

### Common parameters

| Name           | Description                                   | Value |
| -------------- | --------------------------------------------- | ----- |
| `nameOverride` | String to partially override chart's fullname | `""`  |

### Pod labels

| Name             | Description                    | Value |
| ---------------- | ------------------------------ | ----- |
| `extraPodlabels` | Additional labels for the pods | `{}`  |

### Resource requests and limits

| Name                        | Description    | Value   |
| --------------------------- | -------------- | ------- |
| `resources.requests.cpu`    | CPU request    | `100m`  |
| `resources.requests.memory` | Memory request | `128Mi` |
| `resources.limits.cpu`      | CPU limit      | `500m`  |
| `resources.limits.memory`   | Memory limit   | `256Mi` |

### Environment variables

| Name           | Description                                                              | Value                                        |
| -------------- | ------------------------------------------------------------------------ | -------------------------------------------- |
| `env[0].name`  | Name of Routes__0__DownstreamScheme environment variable                 | `Routes__0__DownstreamScheme`                |
| `env[0].value` | Value of Routes__0__DownstreamScheme environment variable                | `http`                                       |
| `env[1].name`  | Name of Routes__0__DownstreamHostAndPorts__0__Host environment variable  | `Routes__0__DownstreamHostAndPorts__0__Host` |
| `env[1].value` | Value of Routes__0__DownstreamHostAndPorts__0__Host environment variable | `gba-hc-converter`                           |
| `env[2].name`  | Name of Routes__0__DownstreamHostAndPorts__0__Port environment variable  | `Routes__0__DownstreamHostAndPorts__0__Port` |
| `env[2].value` | Value of Routes__0__DownstreamHostAndPorts__0__Port environment variable | `3006`                                       |
