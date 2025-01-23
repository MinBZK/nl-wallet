# Setup development environment

## Setup database

The wallet provider requires a Postgres database, named `wallet_provider`, that
is accessible to a user named `postgres` having the password `postgres`. See the
defaults in `wallet_provider/src/settings.rs`.

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

In order to override default settings, create a file named
`wallet_provider.toml` in the same location as `wallet_provider.example.toml` (
which can also be used as a starting point).

Alternatively, see the instructions for setting up a development environment in
the main [`README.md`](../../README.md#configuring-the-development-environment).

Default settings (in `wallet_provider/src/settings.rs`) and settings specified
in `wallet_provider.toml` can both be overriden by environment variables. All
environment variables should be prefixed with `WALLET_PROVIDER__`, e.g.
`WALLET_PROVIDER__SIGNING_PRIVATE_KEY`. Grouped settings can be specified as
follows: `WALLET_PROVIDER__DATABASE__HOST`, where the group name is separated
from the key by a double underscore `__`.

## Generating entity files

Every time the database schema changes, the entities need to be regenerated. For
this, `sea-orm-cli` is used, and can be
installed with:

```bash
cargo install sea-orm-cli
```

From `wallet_core`, run:

```bash
sea-orm-cli generate entity -o wallet_provider/persistence/src/entity --database-url "postgres://localhost/wallet_provider"
```

## Running integration tests

There are database-specific integration test that can be run with:

```bash
cargo test --features db_test
```

There are HSM specific test that can be run with:

```bash
cargo test --features hsm_test
```

## Running the server & retrieving the public keys

During local development, the Wallet Provider can be run with the following
command:

```bash
RUST_LOG=debug cargo run --bin wallet_provider
```

If the log level is at least debug (as above), it will output the public keys
that are derived from the private keys.
This can then be used in development of the Wallet app.
