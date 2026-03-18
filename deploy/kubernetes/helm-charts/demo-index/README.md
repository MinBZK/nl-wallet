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
| `resources.requests.cpu`    | CPU request    | `50m`  |
| `resources.requests.memory` | Memory request | `32Mi` |
| `resources.limits.cpu`      | CPU limit      | `200m` |
| `resources.limits.memory`   | Memory limit   | `64Mi` |

### HTTP route parameters

| Name                    | Description                          | Value  |
| ----------------------- | ------------------------------------ | ------ |
| `httpRoute.enabled`     | Enable or disable the route          | `true` |
| `httpRoute.parentRefs`  | Parent references to the gateway     | `[]`   |
| `httpRoute.hostnames`   | Hostnames for the route              | `[]`   |
| `httpRoute.contextPath` | Optional context path for the route  | `nil`  |
| `httpRoute.labels`      | Additional labels for the route      | `{}`   |
| `httpRoute.annotations` | Additional annotations for the route | `{}`   |

### Container probes

| Name                            | Description                                  | Value   |
| ------------------------------- | -------------------------------------------- | ------- |
| `probes.config.liveness`        | Additional configuration for liveness probe  | `{}`    |
| `probes.config.readiness`       | Additional configuration for readiness probe | `{}`    |
| `probes.config.startup`         | Additional configuration for startup probe   | `{}`    |
| `probes.disableLiveness`        | Disable liveness probe                       | `false` |
| `probes.useLivenessAsReadiness` | Use liveness endpoint for readiness          | `false` |

### Logging

| Name          | Description                                 | Value   |
| ------------- | ------------------------------------------- | ------- |
| `logRequests` | Log request and response headers and bodies | `false` |
