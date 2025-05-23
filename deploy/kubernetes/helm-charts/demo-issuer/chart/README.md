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

| Name               | Description                        | Value          |
| ------------------ | ---------------------------------- | -------------- |
| `image.repository` | Repository for the container image | `nil`          |
| `image.pullPolicy` | Image pull policy                  | `IfNotPresent` |
| `image.tag`        | Image tag                          | `nil`          |

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
| `ingress.hostname`      | Hostname for the ingress               | `nil`   |
| `ingress.contextPath`   | Optional context path for the ingress  | `nil`   |
| `ingress.tlsSecretName` | Name of the TLS secret for the ingress | `nil`   |
| `ingress.labels`        | Additional labels for the ingress      | `{}`    |
| `ingress.annotations`   | Additional annotations for the ingress | `{}`    |

### Resource requests and limits

| Name                        | Description    | Value  |
| --------------------------- | -------------- | ------ |
| `resources.requests.cpu`    | CPU request    | `50m`  |
| `resources.requests.memory` | Memory request | `32Mi` |
| `resources.limits.cpu`      | CPU limit      | `200m` |
| `resources.limits.memory`   | Memory limit   | `64Mi` |

### Urls

| Name                    | Description                              | Value |
| ----------------------- | ---------------------------------------- | ----- |
| `issuanceServerBaseUrl` | Base URL of the issuance server          | `nil` |
| `universalLinkBaseUrl`  | Base URL for universal links             | `nil` |
| `helpBaseUrl`           | Base URL for the help link in wallet web | `nil` |
