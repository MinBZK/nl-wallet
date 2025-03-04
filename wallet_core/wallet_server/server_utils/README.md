# server_utils

This crate contains various tooling and utilities for server side implementations of
OpenID4VCI and/or OpenID4VP.

## Migrate database

This package comes with a separate binary to update the postgres database
tables, i.e. `wallet_server_migrations`.
As an example how to use it, the following snippet can be used to upgrade the
databases in the local development environment, where we use 2 different
instances of the Wallet Server.

```sh
DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/verification_server" cargo run --bin wallet_server_migrations -- fresh
DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/pid_issuer" cargo run --bin wallet_server_migrations -- fresh
```

## Generate entities

```
sea-orm-cli generate entity -o server_utils/src/entity --database-url "postgres://postgres:postgres@127.0.0.1:5432/verification_server"
sea-orm-cli generate entity -o server_utils/src/entity --database-url "postgres://postgres:postgres@127.0.0.1:5432/pid_issuer"
```
