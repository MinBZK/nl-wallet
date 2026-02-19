# Redis Helm Chart

This chart deploys a standalone Redis instance using a StatefulSet and an 
optional PVC.

## Parameters

### Global parameters

| Name                   | Description                  | Value |
| ---------------------- | ---------------------------- | ----- |
| `global.imageRegistry` | Global Docker image registry | `""`  |

### Image parameters

| Name               | Description                        | Value          |
| ------------------ | ---------------------------------- | -------------- |
| `image.repository` | Repository for the container image | `redis`        |
| `image.pullPolicy` | Image pull policy                  | `IfNotPresent` |
| `image.tag`        | Image tag                          | `7.2`          |

### Image pull secrets

| Name               | Description                                  | Value |
| ------------------ | -------------------------------------------- | ----- |
| `imagePullSecrets` | Array of secret names for private registries | `[]`  |

### Common parameters

| Name               | Description                                    | Value |
| ------------------ | ---------------------------------------------- | ----- |
| `fullnameOverride` | String to completely override chart's fullname | `""`  |
| `nameOverride`     | String to partially override chart's fullname  | `""`  |

### Annotations and labels

| Name               | Description                                | Value |
| ------------------ | ------------------------------------------ | ----- |
| `extraAnnotations` | Additional annotations for the statefulset | `{}`  |
| `extraPodLabels`   | Additional labels for the pods             | `{}`  |

### Pod security context

| Name                 | Description                  | Value |
| -------------------- | ---------------------------- | ----- |
| `podSecurityContext` | Security context for the pod | `{}`  |

### Security context

| Name              | Description                        | Value |
| ----------------- | ---------------------------------- | ----- |
| `securityContext` | Security context for the container | `{}`  |

### Resource requests and limits

| Name                        | Description    | Value   |
| --------------------------- | -------------- | ------- |
| `resources.requests.cpu`    | CPU request    | `100m`  |
| `resources.requests.memory` | Memory request | `128Mi` |
| `resources.limits.cpu`      | CPU limit      | `400m`  |
| `resources.limits.memory`   | Memory limit   | `256Mi` |

### Redis server configuration

| Name                       | Description                                                      | Value     |
| -------------------------- | ---------------------------------------------------------------- | --------- |
| `redis.server.port`        | Port used to serve Redis                                         | `6379`    |
| `redis.server.logLevel`    | One of warning, verbose, notice, debug                           | `warning` |
| `redis.server.appendOnly`  | Enable (yes) or disable (no) the append-only file                | `no`      |
| `redis.server.save`        | Save the dataset every N seconds if there are at least M changes | `""`      |
| `redis.server.initialKeys` | Collection of key/values to insert                               | `{}`      |

### Persistence configuration

| Name                       | Description                          | Value   |
| -------------------------- | ------------------------------------ | ------- |
| `persistence.enabled`      | Enable or disable persistent storage | `false` |
| `persistence.storageClass` | Storage class name                   | `""`    |
| `persistence.size`         | Size of the persistent storage       | `1Gi`   |

## Installation

On a locally running single-node cluster, not using a PVC:

```shell
helm install redis . \
  --set extraAnnotations.some-annotation=foo \
  --set redis.server.initialKeys.foo:some_identifier=bar
```

Locally with PVC:

```shell
helm install redis . \
  --set persistence.enabled=true \
  --set persistence.storageClassName=local-path \
  --set extraAnnotations.some-annotation=foo \
  --set redis.server.initialKeys.foo:some_identifier=bar
```

## Connect

You can connect to the redis service using `kubectl port-forward` and either
`redis-cli` or `valkey-cli`:

```shell
# directly on the pod:
kubectl exec redis-0 -- redis-cli get foo:some_identifier
# or from the host using port forwarding:
kubectl port-forward service/redis 6379:6379
redis-cli -h localhost -p 6379 get foo:some_identifier
```
