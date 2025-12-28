# Redis Helm Chart

This chart deploys a standalone Redis instance using a StatefulSet and an 
optional PVC. You can set a list key/values to initialize with, using 
`redisConfig.initialKeys`. Lastly, you can enable network policy and set
a `tierLabel`.

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
| persistence.enabled               | bool   | true              | Enable PVC creation.                                |
| persistence.storageClassName      | string | ""                | StorageClass name for PVC; leave unset for default. |
| persistence.accessModes           | list   | ["ReadWriteOnce"] | PVC access modes.                                   |
| persistence.size                  | string | "1Gi"             | PVC size.                                           |
| networkPolicy.enabled             | bool   | false             | Enable NetworkPolicy.                               |
| networkPolicy.ingressFromSelector | object | {}                | Pod selector labels allowed to access Redis.        |
| tierLabel                         | string | ""                | Optional `tier` label specification.                |

## Example installation

On a locally running single-node cluster, not using a PVC:

```bash
helm install redis charts/redis \
  --set redisConfig.initialKeys[0]="foo:some_key bar"
```

Locally with PVC:

```bash
helm install redis charts/redis \
  --set persistence.enabled=true \
  --set persistence.storageClassName=local-path \
  --set redisConfig.initialKeys[0]="foo:some_key bar"
```

On an enterprisey Kubernetes environment with PVC, NetworkPolicy and tierLabel:

```bash
helm install redis charts/redis \
  --set persistence.enabled=true \
  --set persistence.storageClassName=expensive-storage \
  --set networkPolicy.enabled=true \
  --set networkPolicy.ingressFromSelector.something-this-or-that=allow \
  --set tierLabel=a-silly-tiering-label \
  --set redisConfig.initialKeys[0]="foo:another_key baz"
```
