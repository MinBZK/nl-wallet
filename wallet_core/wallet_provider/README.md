# Setup development environment

## Setup database

The wallet provider requires a Postgres database, named `wallet_provider`, that is accessible to a user
named `postgres` having the password `postgres`. See the defaults in `wallet_provider/src/settings.rs`.

To setup a local database using `psql`:

```sql
create database wallet_provider;
\c wallet_provider
create extension if not exists "uuid-ossp" with schema public;
```

For running the database migrations:

```bash
cargo run --bin wallet_provider_migrations -- fresh
```

This command drops all tables and runs all migrations.

## Settings
Default settings are specified in `src/settings.rs`.

In order to override default settings, create a file named `wallet_provider.toml` in the same location as `wallet_provider.example.toml`.

Override any settings necessary. At a minimum, `certificate_private_key` and `instruction_result_private_key` should be specified. See [Generate signing key](#generate-signing-key) on how to specify that.

Default settings (in `wallet_provider/src/settings.rs`) and settings specified in `wallet_provider.toml` can both be overriden by environment variables. All environment variables should be prefixed with `WALLET_PROVIDER`, e.g. `WALLET_PROVIDER_SIGNING_PRIVATE_KEY`. Grouped settings can be specified as follows: `WALLET_PROVIDER_DATABASE__HOST`, where the group name is separated from the key by a double underscore `__`.

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

## Generate signing keys

The wallet provider expects private signing keys for signing the wallet certificate and for signing instruction results. The private signing keys can be provided via an environment variable (`WALLET_PROVIDER_SIGNING_PRIVATE_KEY` and `WALLET_PROVIDER_INSTRUCTION_RESULT_PRIVATE_KEY`) or via a configuration file named `wallet_provider.toml`. For an example, see `wallet_provider.example.toml`.
Follow the steps below for generating a private key for running the wallet provider locally.

Generating a private key:
```bash
openssl ecparam -name prime256v1 -genkey -noout -out private.ec.key
```

Encode the private key to pkcs8 format:
```bash
openssl pkcs8 -topk8 -nocrypt -in private.ec.key -out private.pem
```

## Generate pin hash salt:

The wallet provider needs a salt for hashing the pin. This can be a random 32-bit value that is base64 encoded (without padding):

```bash
openssl rand 32 | base64 | tr -d '='
```

## Running the server & retrieving the public keys

During local development, the Wallet Provider can be run with the following command:

```bash
RUST_LOG=debug cargo run --bin wallet_provider
```

If the log level is at least debug (as above), it will output the public keys that are derived from the private keys.
This can then be used in development of the Wallet app.
