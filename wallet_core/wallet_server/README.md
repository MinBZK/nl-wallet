# wallet_server

Wallet server consists of three separate binaries that can be used to integrate
with the NL Wallet app:

1. verification_server
2. issuance_server
3. pid_server

It also includes a shared `server_utils` crate that contains various tooling and
utilities for server side implementations of OpenID4VCI and/or OpenID4VP.

All three binaries have their own `migrations` binary to update the postgres
database tables. To migrate for all binaries (including wallet provider) run:

```shell
scripts/migrate-db.sh
```

## Generate entities

To generate the entities for this shared components you have to run our script
against a verification_server.

```shell
scripts/generate-db-entity.sh server_utils
```

To generate the entities for the pid_issuer you have to run:

```shell
scripts/generate-db-entity.sh pid_issuer
```
