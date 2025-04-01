## Parameters

### Global parameters

| Name                   | Description                  | Value |
| ---------------------- | ---------------------------- | ----- |
| `global.imageRegistry` | Global Docker image registry | `""`  |

### Image parameters

| Name               | Description                        | Value          |
| ------------------ | ---------------------------------- | -------------- |
| `image.repository` | Repository for the container image | `nil`          |
| `image.pullPolicy` | Image pull policy                  | `IfNotPresent` |
| `image.tag`        | Image tag                          | `nil`          |

### Image preload parameters

| Name                      | Description                                 | Value          |
| ------------------------- | ------------------------------------------- | -------------- |
| `imagePreload.repository` | Repository for the preload container image  | `nil`          |
| `imagePreload.pullPolicy` | Image pull policy for the preload container | `IfNotPresent` |

### Image GBA Frontend parameters

| Name                          | Description                                      | Value          |
| ----------------------------- | ------------------------------------------------ | -------------- |
| `imageGbaFrontend.repository` | Repository for the GBA Frontend container image  | `nil`          |
| `imageGbaFrontend.pullPolicy` | Image pull policy for the GBA Frontend container | `IfNotPresent` |

### Image pull secrets

| Name               | Description                                  | Value |
| ------------------ | -------------------------------------------- | ----- |
| `imagePullSecrets` | Array of secret names for private registries | `[]`  |

### Service account name

| Name                 | Description                 | Value |
| -------------------- | --------------------------- | ----- |
| `serviceAccountName` | Name of the service account | `nil` |

### Security context

| Name              | Description                         | Value |
| ----------------- | ----------------------------------- | ----- |
| `securityContext` | Security context for the containers | `{}`  |

### GBA HC Converter parameters

| Name                                       | Description                                                    | Value                                 |
| ------------------------------------------ | -------------------------------------------------------------- | ------------------------------------- |
| `gbaHcConverter.replicaCount`              | Number of replicas for the GBA HC Converter                    | `2`                                   |
| `gbaHcConverter.extraPodlabels`            | Additional labels for the pods                                 | `{}`                                  |
| `gbaHcConverter.runMode`                   | Run mode for the GBA HC Converter (e.g., ALL, PRELOADED, GBAV) | `ALL`                                 |
| `gbaHcConverter.envVarNamePreloaded`       | Environment variable name for the preloaded run mode           | `ALL__PRELOADED`                      |
| `gbaHcConverter.envVarNameGbav`            | Environment variable name for the GBAV run mode                | `ALL__GBAV`                           |
| `gbaHcConverter.preloadedXmlPath`          | Path to the preloaded XML files                                | `resources/encrypted-gba-v-responses` |
| `gbaHcConverter.resources.limits.memory`   | Memory limit                                                   | `128Mi`                               |
| `gbaHcConverter.resources.limits.cpu`      | CPU limit                                                      | `400m`                                |
| `gbaHcConverter.resources.requests.memory` | Memory request                                                 | `128Mi`                               |
| `gbaHcConverter.resources.requests.cpu`    | CPU request                                                    | `100m`                                |

### GBA Fetch parameters

| Name                   | Description                    | Value |
| ---------------------- | ------------------------------ | ----- |
| `gbaFetch.frontendUrl` | URL for the GBA Fetch frontend | `nil` |

### GBA CLI Tool parameters

| Name                 | Description                        | Value   |
| -------------------- | ---------------------------------- | ------- |
| `gbaCliTool.enabled` | Enable or disable the GBA CLI Tool | `false` |

### GBA Encrypt Test Data parameters

| Name                                | Description                                          | Value                            |
| ----------------------------------- | ---------------------------------------------------- | -------------------------------- |
| `gbaEncryptTestData.name`           | Name of the GBA Encrypt Test Data cronjob            | `cronjob-encrypt-gba-v-testdata` |
| `gbaEncryptTestData.enabled`        | Enable or disable the GBA Encrypt Test Data          | `false`                          |
| `gbaEncryptTestData.extraPodlabels` | Additional labels for the GBA Encrypt Test Data pods | `{}`                             |

### Host Aliases

| Name          | Description          | Value |
| ------------- | -------------------- | ----- |
| `hostAliases` | List of host aliases | `[]`  |

### Frontend parameters

| Name                                 | Description                                   | Value                |
| ------------------------------------ | --------------------------------------------- | -------------------- |
| `frontend.replicaCount`              | Number of replicas for the GBA Fetch frontend | `1`                  |
| `frontend.name`                      | Name of the GBA Fetch frontend application    | `gba-fetch-frontend` |
| `frontend.extraPodlabels`            | Additional labels for the frontend pods       | `{}`                 |
| `frontend.ingress.className`         | Ingress class name for the frontend           | `nginx`              |
| `frontend.ingress.secretName`        | Secret name for the ingress TLS configuration | `nil`                |
| `frontend.resources.limits.memory`   | Memory limit for the frontend container       | `128Mi`              |
| `frontend.resources.limits.cpu`      | CPU limit for the frontend container          | `100m`               |
| `frontend.resources.requests.memory` | Memory request for the frontend container     | `128Mi`              |
| `frontend.resources.requests.cpu`    | CPU request for the frontend container        | `50m`                |

### Preload GBA Pod parameters

| Name                           | Description                               | Value                    |
| ------------------------------ | ----------------------------------------- | ------------------------ |
| `preloadGbaPod.name`           | Name of the Preload GBA Pod               | `preload-gba-v-data-pod` |
| `preloadGbaPod.extraPodlabels` | Additional labels for the Preload GBA Pod | `{}`                     |

