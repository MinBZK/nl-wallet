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

### Pod security context

| Name                 | Description                  | Value |
| -------------------- | ---------------------------- | ----- |
| `podSecurityContext` | Security context for the pod | `{}`  |

### Security context

| Name              | Description                        | Value |
| ----------------- | ---------------------------------- | ----- |
| `securityContext` | Security context for the container | `{}`  |

### HTTP route parameters

| Name                    | Description                          | Value  |
| ----------------------- | ------------------------------------ | ------ |
| `httpRoute.enabled`     | Enable or disable the route          | `true` |
| `httpRoute.parentRefs`  | Parent references to the gateway     | `[]`   |
| `httpRoute.hostnames`   | Hostnames for the route              | `[]`   |
| `httpRoute.contextPath` | Optional context path for the route  | `nil`  |
| `httpRoute.labels`      | Additional labels for the route      | `{}`   |
| `httpRoute.annotations` | Additional annotations for the route | `{}`   |

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

### Resource requests and limits

| Name                        | Description                                                       | Value   |
| --------------------------- | ----------------------------------------------------------------- | ------- |
| `resources.requests.cpu`    | CPU request                                                       | `50m`   |
| `resources.requests.memory` | Memory request                                                    | `64Mi`  |
| `resources.limits.cpu`      | CPU limit                                                         | `200m`  |
| `resources.limits.memory`   | Memory limit                                                      | `128Mi` |
| `database.secretName`       | The secret name that contains the connection url for the database | `nil`   |

### Urls

| Name                   | Description                  | Value |
| ---------------------- | ---------------------------- | ----- |
| `universalLinkBaseUrl` | Base URL for universal links | `nil` |

### Migration parameters

| Name               | Description                             | Value  |
| ------------------ | --------------------------------------- | ------ |
| `migration.labels` | Additional labels for the migration job | `{}`   |
| `migration.reset`  | Enable reset cron job                   | `true` |

### Persistence parameters

| Name                           | Description                                        | Value |
| ------------------------------ | -------------------------------------------------- | ----- |
| `persistence.storageClassName` | Storage class name for the persistent volume claim | `nfs` |
