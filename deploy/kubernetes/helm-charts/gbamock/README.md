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

| Name               | Description                        | Value                                                       |
| ------------------ | ---------------------------------- | ----------------------------------------------------------- |
| `image.repository` | Repository for the container image | `ghcr-io-proxy/brp-api/haal-centraal-brp-bevragen-gba-mock` |
| `image.pullPolicy` | Image pull policy                  | `IfNotPresent`                                              |
| `image.tag`        | Image tag                          | `latest`                                                    |

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
| `resources.requests.cpu`    | CPU request    | `50m`   |
| `resources.requests.memory` | Memory request | `64Mi`  |
| `resources.limits.cpu`      | CPU limit      | `200m`  |
| `resources.limits.memory`   | Memory limit   | `128Mi` |

### GbaMock parameters

| Name              | Description                              | Value  |
| ----------------- | ---------------------------------------- | ------ |
| `gbaMock.enabled` | Enable or disable the GbaMock deployment | `true` |

