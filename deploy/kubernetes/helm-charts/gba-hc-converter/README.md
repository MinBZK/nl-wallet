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

### Network Policy configuration

| Name                    | Description                               | Value |
| ----------------------- | ----------------------------------------- | ----- |
| `networkPolicy.ingress` | Ingress rules for GBA converter pods pods | `[]`  |

### GBA HC Converter parameters

| Name                                       | Description                                                    | Value                                 |
| ------------------------------------------ | -------------------------------------------------------------- | ------------------------------------- |
| `gbaHcConverter.replicaCount`              | Number of replicas for the GBA HC Converter                    | `2`                                   |
| `gbaHcConverter.extraAnnotations`          | Additional annotations for the BA HC Converter deployment      | `{}`                                  |
| `gbaHcConverter.extraPodLabels`            | Additional labels for the BA HC Converter pods                 | `{}`                                  |
| `gbaHcConverter.runMode`                   | Run mode for the GBA HC Converter (e.g., ALL, PRELOADED, GBAV) | `PRELOADED`                           |
| `gbaHcConverter.envVarNamePreloaded`       | Environment variable name for the preloaded run mode           | `PRELOADED`                           |
| `gbaHcConverter.envVarNameGbav`            | Environment variable name for the GBAV run mode                | `ALL__GBAV`                           |
| `gbaHcConverter.preloadedXmlPath`          | Path to the preloaded XML files                                | `resources/encrypted-gba-v-responses` |
| `gbaHcConverter.resources.requests.cpu`    | CPU request                                                    | `50m`                                 |
| `gbaHcConverter.resources.requests.memory` | Memory request                                                 | `64Mi`                                |
| `gbaHcConverter.resources.limits.cpu`      | CPU limit                                                      | `200m`                                |
| `gbaHcConverter.resources.limits.memory`   | Memory limit                                                   | `128Mi`                               |

### GBA CLI Tool parameters

| Name                     | Description                                 | Value   |
| ------------------------ | ------------------------------------------- | ------- |
| `gbaCliTool.enabled`     | Enable or disable the GBA CLI Tool          | `false` |
| `gbaCliTool.useRijksweb` | Enable or disable the use of Rijksweb proxy | `false` |

### GBA Encrypt Test Data parameters

| Name                                  | Description                                                  | Value                    |
| ------------------------------------- | ------------------------------------------------------------ | ------------------------ |
| `gbaEncryptTestData.name`             | Name of the GBA Encrypt Test Data cronjob                    | `encrypt-gba-v-testdata` |
| `gbaEncryptTestData.enabled`          | Enable or disable the GBA Encrypt Test Data                  | `false`                  |
| `gbaEncryptTestData.extraAnnotations` | Additional annotations for the GBA Encrypt Test Data cronjob | `{}`                     |
| `gbaEncryptTestData.extraPodLabels`   | Additional labels for the GBA Encrypt Test Data pods         | `{}`                     |

### Host Aliases

| Name          | Description          | Value |
| ------------- | -------------------- | ----- |
| `hostAliases` | List of host aliases | `[]`  |

### Frontend parameters

| Name                                 | Description                                        | Value                |
| ------------------------------------ | -------------------------------------------------- | -------------------- |
| `frontend.replicaCount`              | Number of replicas for the GBA Fetch frontend      | `1`                  |
| `frontend.name`                      | Name of the GBA Fetch frontend application         | `gba-fetch-frontend` |
| `frontend.extraAnnotations`          | Additional annotations for the frontend deployment | `{}`                 |
| `frontend.extraPodLabels`            | Additional labels for the frontend pods            | `{}`                 |
| `frontend.ingress.className`         | Ingress class name for the frontend                | `nginx`              |
| `frontend.ingress.hostname`          | Ingress hostname for the frontend                  | `nil`                |
| `frontend.ingress.tlsSecretName`     | Secret name for the ingress TLS configuration      | `nil`                |
| `frontend.resources.requests.cpu`    | CPU request for the frontend container             | `50m`                |
| `frontend.resources.requests.memory` | Memory request for the frontend container          | `64Mi`               |
| `frontend.resources.limits.cpu`      | CPU limit for the frontend container               | `200m`               |
| `frontend.resources.limits.memory`   | Memory limit for the frontend container            | `128Mi`              |

### Preload GBA deployement parameters

| Name                          | Description                                           | Value                  |
| ----------------------------- | ----------------------------------------------------- | ---------------------- |
| `preloadGba.name`             | Name of the Preload GBA Deployment                    | `preload-gba-v-data`   |
| `preloadGba.extraAnnotations` | Additional annotations for the Preload GBA Deployment | `{}`                   |
| `preloadGba.extraPodLabels`   | Additional labels for the Preload GBA Pod             | `{}`                   |
| `preloadGba.volumeClaimName`  | Name of the Preload GBA Volume Claim                  | `preloaded-gba-v-data` |

### xml files

| Name       | Description                                          | Value |
| ---------- | ---------------------------------------------------- | ----- |
| `xmlFiles` | List of XML files to be used in the GBA HC Converter | `{}`  |

### zoekXmlFiles

| Name           | Description                                          | Value |
| -------------- | ---------------------------------------------------- | ----- |
| `zoekXmlFiles` | List of XML files to be used in the GBA HC Converter | `{}`  |


