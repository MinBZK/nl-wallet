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

### Audit Log Image migration parameters

| Name                                 | Description                                   | Value          |
| ------------------------------------ | --------------------------------------------- | -------------- |
| `imageAuditLogMigrations.repository` | Repository for the migration container image  | `nil`          |
| `imageAuditLogMigrations.pullPolicy` | Image pull policy for the migration container | `IfNotPresent` |

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
| `serviceAccountName` | Name of the service account        | `""`  |
| `podSecurityContext` | Security context for the pod       | `{}`  |
| `securityContext`    | Security context for the container | `{}`  |

### Annotations and labels

| Name               | Description                               | Value |
| ------------------ | ----------------------------------------- | ----- |
| `extraAnnotations` | Additional annotations for the deployment | `{}`  |
| `extraPodLabels`   | Additional labels for the pods            | `{}`  |

### Resource requests and limits

| Name                        | Description    | Value  |
| --------------------------- | -------------- | ------ |
| `resources.requests.cpu`    | CPU request    | `100m` |
| `resources.requests.memory` | Memory request | `32Mi` |
| `resources.limits.cpu`      | CPU limit      | `400m` |
| `resources.limits.memory`   | Memory limit   | `64Mi` |

### HTTP route parameters

| Name                                  | Description                                          | Value  |
| ------------------------------------- | ---------------------------------------------------- | ------ |
| `httpRoute.enabled`                   | Enable or disable the route                          | `true` |
| `httpRoute.parentRefs`                | Parent references to the gateway                     | `[]`   |
| `httpRoute.hostnames`                 | Hostnames for the route                              | `[]`   |
| `httpRoute.contextPath`               | Optional context path for the route                  | `nil`  |
| `httpRoute.labels`                    | Additional labels for the route                      | `{}`   |
| `httpRoute.annotations`               | Additional annotations for the route                 | `{}`   |
| `httpRoute.nginxClientSettingsPolicy` | Nginx specific client settings policy for this route | `{}`   |

### HTTP route internal parameters

| Name                            | Description                          | Value  |
| ------------------------------- | ------------------------------------ | ------ |
| `httpRouteInternal.enabled`     | Enable or disable the route          | `true` |
| `httpRouteInternal.parentRefs`  | Parent references to the gateway     | `[]`   |
| `httpRouteInternal.hostnames`   | Hostnames for the route              | `[]`   |
| `httpRouteInternal.labels`      | Additional labels for the route      | `{}`   |
| `httpRouteInternal.annotations` | Additional annotations for the route | `{}`   |

### Container probes

| Name                            | Description                                  | Value   |
| ------------------------------- | -------------------------------------------- | ------- |
| `probes.config.liveness`        | Additional configuration for liveness probe  | `{}`    |
| `probes.config.readiness`       | Additional configuration for readiness probe | `{}`    |
| `probes.config.startup`         | Additional configuration for startup probe   | `{}`    |
| `probes.disableLiveness`        | Disable liveness probe                       | `false` |
| `probes.useLivenessAsReadiness` | Use liveness endpoint for readiness          | `false` |

### Database parameters

| Name                  | Description                                                       | Value |
| --------------------- | ----------------------------------------------------------------- | ----- |
| `database.secretName` | The secret name that contains the connection url for the database | `nil` |

### Audit Log Database parameters

| Name                  | Description                                                       | Value |
| --------------------- | ----------------------------------------------------------------- | ----- |
| `auditLog.secretName` | The secret name that contains the connection url for the database | `nil` |

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

| Name                                  | Description                                                | Value               |
| ------------------------------------- | ---------------------------------------------------------- | ------------------- |
| `appIdentifier`                       | Application identifier                                     | `nil`               |
| `flagsRefreshDelayInSeconds`          | Interval in seconds of background job that refreshes flags | `nil`               |
| `recoveryCodePaths.urn:eudi:pid:nl:1` | Recovery code path for default PID attestation             | `["recovery_code"]` |
| `revokeSolutionEnabled`               | Whether the revoke-solution API is enabled                 | `false`             |

### WUA status list parameters

| Name                            | Description                                                | Value |
| ------------------------------- | ---------------------------------------------------------- | ----- |
| `wuaStatusList.baseUrl`         | WUA status list base url that will be encoded as iss claim | `nil` |
| `wuaStatusList.certificate`     | WUA status list certificate                                | `nil` |
| `wuaStatusList.volumeClaimName` | Name of PVC where the WUA status lists are published       | `nil` |
| `wuaStatusList.volumeClaimPath` | Path in the PVC where the WUA status lists are published   | `nil` |

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
| `logSidecar.securityContext`           | Security context for the log sidecar    | `{}`           |
