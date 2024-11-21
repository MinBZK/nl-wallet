# Wallet Server

The Wallet Server is a supporting server for issuers and/or verifiers, it
implements the protocol(s) that the NL Wallet supports and offers a REST API
towards the requester.

## Migrate database

The Wallet Server comes with a separate binary to update the postgres database
tables, i.e. `wallet_server_migration`.
As an example how to use it, the following snippet can be used to upgrade the
databases in the local development environment, where we use 2 different
instances of the Wallet Server.

```sh
DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/verification_server" cargo run --bin wallet_server_migration -- fresh
DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/pid_issuer" cargo run --bin wallet_server_migration -- fresh
```

## Generate entities

```
sea-orm-cli generate entity -o wallet_server/src/entity --database-url "postgres://postgres:postgres@127.0.0.1:5432/verification_server"
sea-orm-cli generate entity -o wallet_server/src/entity --database-url "postgres://postgres:postgres@127.0.0.1:5432/pid_issuer"
```
