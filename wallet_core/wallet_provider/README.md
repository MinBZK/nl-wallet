# Setup development environment

## Setup database

The wallet provider requires a Postgres database, named `wallet_provider`, that is accessible to a user
named `postgres` having the password `postgres`. See `wallet_provider/config.default.toml`.

To setup a local database using `psql`:

```sql
create database wallet_provider;
\c wallet_provider
create extension if not exists "uuid-ossp" with schema public;
```

For running the database migrations:

```bash
cargo run --bin wallet_provider_migrations --features wallet_provider_migrations -- fresh
```

This command drops all tables and runs all migrations.

## Settings
Default settings are specified in `src/settings.rs`.

In order to override default settings, create a file named `config.toml` in the same location as `config.example.toml`.

Override any settings necessary. At a minimum, `signing_private_key` should be specified. See [Generate signing key](#generate-signing-key) on how to specify that.

Default settings and settings specified in `config.toml` can both be overriden by environment variables. All environment variables should be prefixed with `WALLET_PROVIDER`, e.g. `WALLET_PROVIDER_SIGNING_PRIVATE_KEY`. Grouped settings can be specified as follows: `WALLET_PROVIDER_DATABASE__HOST`, where the group name is separated from the key by a double underscore `__`.

## Generating entity files

Every time the database schema changes, the entities need to be regenerated. For this, `sea-orm-cli` is used, and can be
installed with:

```bash
cargo install sea-orm-cli
```

From `wallet_core`, run:

```bash
sea-orm-cli generate entity -o wallet_provider/persistence/src/entity --database-url "postgres://localhost/wallet_provider"
```

## Running integration tests

```bash
cargo test --features db_test --test '*'
```

## Generate signing key

The wallet provider expects a private signing key of which the corresponding public key is used to sign messages from the provider to the wallet. The private signing key can be provided via an environment variable (`WALLET_PROVIDER_SIGNING_PRIVATE_KEY`) or via a configuration file named `config.toml`. For an example, see `config.example.toml`.
Follow the steps below for generating a private key for running the wallet provider locally.

Generating a private key:
```bash
openssl ecparam -name prime256v1 -genkey -noout -out private.ec.key
```

Encode the private key to pkcs8 format:
```bash
openssl pkcs8 -topk8 -nocrypt -in private.ec.key -out private.pem
```
