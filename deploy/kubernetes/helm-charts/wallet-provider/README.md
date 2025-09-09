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

### Annotations and labels

| Name               | Description                               | Value |
| ------------------ | ----------------------------------------- | ----- |
| `extraAnnotations` | Additional annotations for the deployment | `{}`  |
| `extraPodLabels`   | Additional labels for the pods            | `{}`  |

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

### HSM parameters

| Name                          | Description                                              | Value                  |
| ----------------------------- | -------------------------------------------------------- | ---------------------- |
| `hsm.maxSessions`             | Maximum HSM sessions                                     | `10`                   |
| `hsm.maxSessionLifeTimeInSec` | The maximum lifetime of a HSM session in seconds         | `900`                  |
| `hsm.configMapName`           | Name to the the ConfigMap containing the hsm config file | `nl-wallet-hsm-pkcs11` |
| `hsm.configMapKey`            | Key of the the ConfigMap containing the hsm config file  | `cs_pkcs11_R3.cfg`     |

### Android parameters

| Name                                 | Description                                                             | Value                               |
| ------------------------------------ | ----------------------------------------------------------------------- | ----------------------------------- |
| `android.rootPublicKeys`             | Android root public keys                                                | `nil`                               |
| `android.playstoreCertificateHashes` | Google Play Store certificate hashes                                    | `nil`                               |
| `android.allowSideLoading`           | Allow installing apps from sources other than the Play Store            | `false`                             |
| `android.serviceAccount.secretName`  | Name to the the Secret containing the Google Cloud service account file | `nl-wallet-gcloud-service-account`  |
| `android.serviceAccount.secretKey`   | Key of the the Secret containing the Google Cloud service account file  | `google-cloud-service-account.json` |

### iOS parameters

| Name                   | Description             | Value |
| ---------------------- | ----------------------- | ----- |
| `ios.rootCertificates` | iOS root certificates   | `nil` |
| `ios.teamIdentifier`   | Team identifier for iOS | `nil` |

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

### Log sidecar

| Name                                   | Description                             | Value          |
| -------------------------------------- | --------------------------------------- | -------------- |
| `logSidecar.image.repository`          | Repository for the log container image  | `nil`          |
| `logSidecar.image.pullPolicy`          | Image pull policy for the log container | `IfNotPresent` |
| `logSidecar.image.tag`                 | Image tag for the log container         | `nil`          |
| `logSidecar.resources.requests.cpu`    | CPU request of log sidecar              | `100m`         |
| `logSidecar.resources.requests.memory` | Memory request of log sidecar           | `64Mi`         |
| `logSidecar.resources.limits.cpu`      | CPU limit of log sidecar                | `400m`         |
| `logSidecar.resources.limits.memory`   | Memory limit of log sidecar             | `128Mi`        |
