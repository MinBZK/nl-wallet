## Parameters

### Global parameters

| Name                   | Description                  | Value |
| ---------------------- | ---------------------------- | ----- |
| `global.imageRegistry` | Global Docker image registry | `""`  |

### Annotations

| Name          | Description                     | Value |
| ------------- | ------------------------------- | ----- |
| `annotations` | Key-value pairs for annotations | `{}`  |

### Database configuration

| Name            | Description                                               | Value |
| --------------- | --------------------------------------------------------- | ----- |
| `database.name` | Name of the default database                              | `app` |
| `name`          | String to override Cluster name, default is database name | `nil` |

### Common parameters

| Name           | Description        | Value |
| -------------- | ------------------ | ----- |
| `replicaCount` | Number of replicas | `2`   |

### Pod labels

| Name             | Description                    | Value |
| ---------------- | ------------------------------ | ----- |
| `extraPodlabels` | Additional labels for the pods | `{}`  |

### Image configuration

| Name               | Description                     | Value                                     |
| ------------------ | ------------------------------- | ----------------------------------------- |
| `image.repository` | Repository for the Docker image | `ghcr-io-proxy/cloudnative-pg/postgresql` |
| `image.tag`        | Tag for the Docker image        | `16.7-bookworm`                           |

### PgBouncer Image configuration

| Name                        | Description                               | Value                                    |
| --------------------------- | ----------------------------------------- | ---------------------------------------- |
| `imagePgBouncer.repository` | Repository for the PgBouncer Docker image | `ghcr-io-proxy/cloudnative-pg/pgbouncer` |
| `imagePgBouncer.tag`        | Tag for the PgBouncer Docker image        | `1.24.0`                                 |

### Persistence configuration

| Name                       | Description                       | Value          |
| -------------------------- | --------------------------------- | -------------- |
| `persistence.storageClass` | Storage class for the persistence | `standard-csi` |
| `persistence.size`         | Size of the persistent storage    | `5Gi`          |

### Resource configuration

| Name                        | Description                      | Value   |
| --------------------------- | -------------------------------- | ------- |
| `resources.requests.cpu`    | CPU request for the container    | `400m`  |
| `resources.requests.memory` | Memory request for the container | `384Mi` |
| `resources.limits.cpu`      | CPU limit for the container      | `1000m` |
| `resources.limits.memory`   | Memory limit for the container   | `512Mi` |

### PostgreSQL parameters

| Name                         | Description                                                         | Value   |
| ---------------------------- | ------------------------------------------------------------------- | ------- |
| `parameters.max_connections` | Maximum number of connections                                       | `500`   |
| `parameters.shared_buffers`  | Amount of memory the database server uses for shared memory buffers | `256MB` |
| `parameters.log_statement`   | Sets the type of SQL statements logged                              | `none`  |
| `parameters.wal_keep_size`   | Size of WAL files to keep for standby servers                       | `512MB` |
| `parameters.archive_timeout` | Maximum time to wait before performing a WAL file switch            | `5min`  |

### User configuration

| Name                        | Description                                  | Value      |
| --------------------------- | -------------------------------------------- | ---------- |
| `users.migrator.name`       | Name of the migrator user                    | `migrator` |
| `users.migrator.nameSecret` | Secret name for the migrator user's password | `nil`      |

### Backup configuration

| Name                                        | Description                                            | Value         |
| ------------------------------------------- | ------------------------------------------------------ | ------------- |
| `backup.enabled`                            | Enable or disable backups                              | `false`       |
| `backup.schedule`                           | Cron schedule for backups                              | `0 0 0 * * 0` |
| `backup.endpointURL`                        | Endpoint URL for the backup storage                    | `nil`         |
| `backup.destinationPath`                    | Destination path for the backups                       | `nil`         |
| `backup.s3Credentials.accessKeyId.key`      | Key for the S3 access key ID                           | `user`        |
| `backup.s3Credentials.accessKeyId.name`     | Name of the secret containing the S3 access key ID     | `nil`         |
| `backup.s3Credentials.secretAccessKey.key`  | Key for the S3 secret access key                       | `password`    |
| `backup.s3Credentials.secretAccessKey.name` | Name of the secret containing the S3 secret access key | `nil`         |
| `backup.wall.maxParallel`                   | Maximum number of parallel WAL file transfers          | `8`           |
| `backup.retentionPolicy`                    | Retention policy for the backups                       | `7d`          |
| `backup.serverName`                         | Name of the backup server                              | `nil`         |
| `backup.recovery`                           | Enable or disable recovery                             | `false`       |
| `backup.recoveryServerName`                 | Name of the recovery server                            | `nil`         |
| `backup.targetTime`                         | Target time for point-in-time recovery                 | `nil`         |

### Network Policy configuration

| Name                                          | Description                             | Value       |
| --------------------------------------------- | --------------------------------------- | ----------- |
| `networkPolicy.matchComponent`                | Component to match for network policies | `pgbouncer` |
| `networkPolicy.minio.podSelector.matchLabels` | Labels to match for Minio pods          | `{}`        |
| `networkPolicy.postgres.ingress`              | Ingress rules for PostgreSQL pods       | `[]`        |

### Pooling configuration

| Name              | Description               | Value  |
| ----------------- | ------------------------- | ------ |
| `pooling.enabled` | Enable or disable pooling | `true` |

### PgPools configuration

| Name      | Description                               | Value |
| --------- | ----------------------------------------- | ----- |
| `pgPools` | List of pool configurations for PgBouncer | `[]`  |

