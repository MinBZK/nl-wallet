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

| Name                    | Description                                  | Value    |
| ----------------------- | -------------------------------------------- | -------- |
| `imagePullSecrets`      | Array of secret names for private registries | `[]`     |
| `ingress.enabled`       | Enable or disable the ingress                | `true`   |
| `ingress.className`     | Ingress class name                           | `nginx`  |
| `ingress.cspScriptSrc`  | Content Security Policy script source        | `'none'` |
| `ingress.cspStyleSrc`   | Content Security Policy style source         | `'none'` |
| `ingress.tlsSecretName` | Name of the TLS secret for the ingress       | `nil`    |
| `ingress.labels`        | Additional labels for the ingress            | `{}`     |

### Resource requests and limits

| Name                        | Description    | Value   |
| --------------------------- | -------------- | ------- |
| `resources.limits.memory`   | Memory limit   | `512Mi` |
| `resources.limits.cpu`      | CPU limit      | `500m`  |
| `resources.requests.memory` | Memory request | `512Mi` |
| `resources.requests.cpu`    | CPU request    | `300m`  |

### Wallet Server parameters

| Name                       | Description                        | Value |
| -------------------------- | ---------------------------------- | ----- |
| `walletServer.hostname`    | Hostname for the wallet server     | `nil` |
| `walletServer.contextPath` | Context path for the wallet server | `nil` |

### Demo Relying Party parameters

| Name                                 | Description                              | Value |
| ------------------------------------ | ---------------------------------------- | ----- |
| `demoRelyingParty.hostname`          | Hostname for the demo relying party      | `nil` |
| `demoRelyingParty.contextPath`       | Context path for the demo relying party  | `nil` |
| `demoRelyingParty.walletWebFilename` | Filename for the wallet web              | `nil` |
| `demoRelyingParty.walletWebSha256`   | SHA-256 checksum for the wallet web file | `nil` |

