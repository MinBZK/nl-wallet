# Redis Helm Chart

This chart deploys a standalone Redis instance using a StatefulSet and an 
optional PVC. You can set a list of key/values to initialize with, using
`redisConfig.initialKeys`. Lastly, you can enable a network policy and
optionally set a `tierLabel`.

## Values

| Key                               | Type   | Default           | Description                                         |
|-----------------------------------|--------|-------------------|-----------------------------------------------------|
| nameOverride                      | string | ""                | Override chart name, if used, set repository too.   |
| fullnameOverride                  | string | ""                | Override full release name.                         |
| replicaCount                      | int    | 1                 | Number of replicas.                                 |
| minReadySeconds                   | int    | 2                 | Minimum ready seconds for StatefulSet.              |
| image.repository                  | string | ""                | Redis image repository; defaults to `Chart.Name`    |
| image.tag                         | string | ""                | Redis image tag; defaults to `Chart.appVersion`     |
| image.pullPolicy                  | string | "IfNotPresent"    | Image pull policy.                                  |
| imagePullSecrets                  | list   | []                | Image pull secrets.                                 |
| service.port                      | int    | 6379              | Service port.                                       |
| resources                         | object | see values.yaml   | Container resource requests/limits.                 |
| securityContext.runAsUser         | int    | null              | Run container as specific UID.                      |
| redisConfig.appendonly            | string | "yes"             | Enables Append-Only File persistence.               |
| redisConfig.save                  | string | ""                | "seconds changes" pairs for snapshot intervals.     |
| redisConfig.loglevel              | string | "warning"         | Redis log level.                                    |
| redisConfig.initialKeys           | list   | []                | Do `redis-cli set` for each `"key value"`.          |
| persistence.enabled               | bool   | false             | Enable PVC creation.                                |
| persistence.storageClassName      | string | ""                | StorageClass name for PVC; leave unset for default. |
| persistence.accessModes           | list   | ["ReadWriteOnce"] | PVC access modes.                                   |
| persistence.size                  | string | "1Gi"             | PVC size.                                           |
| networkPolicy.enabled             | bool   | false             | Enable NetworkPolicy.                               |
| networkPolicy.ingressFromSelector | object | {}                | Pod selector labels allowed to access Redis.        |
| tierLabel                         | string | ""                | Optional `tier` label specification.                |

## Installation

On a locally running single-node cluster, not using a PVC:

```shell
helm install redis charts/redis \
  --set redisConfig.initialKeys[0]="foo:some_key bar"
```

Locally with PVC:

```shell
helm install redis charts/redis \
  --set persistence.enabled=true \
  --set persistence.storageClassName=local-path \
  --set redisConfig.initialKeys[0]="foo:some_key bar"
```

On an enterprisey Kubernetes environment with PVC, NetworkPolicy and tierLabel:

```shell
helm install redis charts/redis \
  --set persistence.enabled=true \
  --set persistence.storageClassName=expensive-storage \
  --set networkPolicy.enabled=true \
  --set networkPolicy.ingressFromSelector.something-this-or-that=allow \
  --set tierLabel=a-silly-tiering-label \
  --set redisConfig.initialKeys[0]="foo:another_key baz"
```

## Connect

You can connect to the redis service using `kubectl port-forward` and either
`redis-cli` or `valkey-cli`:

```shell
# directly on the pod:
kubectl exec redis-0 -- redis-cli get foo:some_key
# or from the host using port forwarding:
kubectl port-forward service/redis 6379:6379
redis-cli -h localhost -p 6379 get foo:some_key
```
