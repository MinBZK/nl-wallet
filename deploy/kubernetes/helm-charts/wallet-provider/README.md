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

### Service account name

| Name                 | Description                 | Value |
| -------------------- | --------------------------- | ----- |
| `serviceAccountName` | Name of the service account | `nil` |

### Security context

| Name              | Description                         | Value |
| ----------------- | ----------------------------------- | ----- |
| `securityContext` | Security context for the containers | `{}`  |

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

| Name                        | Description                                                       | Value  |
| --------------------------- | ----------------------------------------------------------------- | ------ |
| `resources.requests.cpu`    | CPU request                                                       | `100m` |
| `resources.requests.memory` | Memory request                                                    | `32Mi` |
| `resources.limits.cpu`      | CPU limit                                                         | `400m` |
| `resources.limits.memory`   | Memory limit                                                      | `64Mi` |
| `database.secretName`       | The secret name that contains the connection url for the database | `nil`  |

### Volumes

| Name                           | Description                                | Value                                    |
| ------------------------------ | ------------------------------------------ | ---------------------------------------- |
| `volumes[0].name`              | Name of the first volume                   | `pkcs11-config-volume`                   |
| `volumes[0].configMap.name`    | Name of the ConfigMap for the first volume | `nl-wallet-hsm-pkcs11`                   |
| `volumes[1].name`              | Name of the second volume                  | `wallet-provider-gcloud-service-account` |
| `volumes[1].secret.secretName` | Name of the secret for the second volume   | `nl-wallet-gcloud-service-account`       |

### Volume mounts

| Name                        | Description                             | Value                                    |
| --------------------------- | --------------------------------------- | ---------------------------------------- |
| `volumeMounts[0].name`      | Name of the first volume mount          | `pkcs11-config-volume`                   |
| `volumeMounts[0].mountPath` | Mount path for the first volume mount   | `/cs_pkcs11_R3.cfg`                      |
| `volumeMounts[0].subPath`   | Sub-path within the first volume mount  | `cs_pkcs11_R3.cfg`                       |
| `volumeMounts[1].name`      | Name of the second volume mount         | `wallet-provider-gcloud-service-account` |
| `volumeMounts[1].mountPath` | Mount path for the second volume mount  | `/google-cloud-service-account.json`     |
| `volumeMounts[1].subPath`   | Sub-path within the second volume mount | `google-cloud-service-account.json`      |

### Environment variables from ConfigMaps or Secrets

| Name                           | Description                                               | Value                    |
| ------------------------------ | --------------------------------------------------------- | ------------------------ |
| `envFrom[0].configMapRef.name` | Name of the ConfigMap for the environment variable source | `wallet-provider-config` |

### HSM parameters

| Name              | Description          | Value |
| ----------------- | -------------------- | ----- |
| `hsm.maxSessions` | Maximum HSM sessions | `10`  |

### ConfigMap parameters

| Name                                   | Description                                                  | Value   |
| -------------------------------------- | ------------------------------------------------------------ | ------- |
| `configmap.iosRootCertificates`        | iOS root certificates override                               | `nil`   |
| `configmap.rootPublicKeys`             | Android root public keys                                     | `nil`   |
| `configmap.playstoreCertificateHashes` | Google Play Store certificate hashes                         | `nil`   |
| `configmap.allowSideLoading`           | Allow installing apps from sources other than the Play Store | `false` |

### iOS parameters

| Name                 | Description             | Value |
| -------------------- | ----------------------- | ----- |
| `ios.teamIdentifier` | Team identifier for iOS | `nil` |

### Application parameters

| Name            | Description            | Value |
| --------------- | ---------------------- | ----- |
| `appIdentifier` | Application identifier | `nil` |

### Migration parameters

| Name               | Description                             | Value  |
| ------------------ | --------------------------------------- | ------ |
| `migration.labels` | Additional labels for the migration job | `{}`   |
| `migration.reset`  | Enable reset cron job                   | `true` |

### Service monitor

| Name                     | Description            | Value  |
| ------------------------ | ---------------------- | ------ |
| `serviceMonitor.enabled` | Enable service monitor | `true` |
