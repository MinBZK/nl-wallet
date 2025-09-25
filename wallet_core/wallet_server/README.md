# wallet_server

Wallet server consists of three separate binaries that can be used to integrate
with the NL Wallet app:

1. verification_server
2. issuance_server
3. pid_server

It also includes a shared `server_utils` crate that contains shared settings,
persistent session state for the `openid4vc` crate and server setup.

All three binaries have their own `migrations` binary to update the postgres
database tables. To migrate for all binaries (including wallet provider) run:

```shell
"$(git rev-parse --show-toplevel)"/scripts/migrate-db.sh
```

## Generate entities

To generate the entities for this shared components you have to run our script
against a verification_server.

```shell
"$(git rev-parse --show-toplevel)"/scripts/generate-db-entity.sh server_utils
```
