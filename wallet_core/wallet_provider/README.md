# Setup development environment

## Setup database

The wallet provider requires a Postgres database, named `wallet_provider`. By
default it uses localhost with `postgres` as username and password. You can
change this via the settings file and/or `scripts/.env` for `setup-devenv.sh`.

To setup a local database using `psql`:

```sql
create database wallet_provider;
create database wallet_provider_audit_log;
```

For running the database migrations:

```bash
DATABASE_URL="postgres://$DB_USERNAME:$DB_PASSWORD@$PGHOST:$PGPORT/wallet_provider" \
cargo run --bin wallet_provider_migrations -- fresh

DATABASE_URL="postgres://$DB_USERNAME:$DB_PASSWORD@$PGHOST:$PGPORT/wallet_provider_audit_log" \
cargo run --bin audit_log_migrations -- fresh
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
follows: `WALLET_PROVIDER__DATABASE__URL`, where the group name is separated
from the key by a double underscore `__`.

## Generating entity files

Every time the database schema changes, the entities need to be regenerated. For
this we have created a script that uses the `sea-orm-cli`.

```shell
scripts/generate-db-entity.sh wallet_provider
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

## Android registration without Google Play Integrity (de-Googled devices)

By default, an Android wallet registration must provide both a hardware key
attestation certificate chain **and** a Google Play Integrity token. The Play
Integrity token requires Google Play services and, in practice, a Google
account, which excludes de-Googled devices (e.g. GrapheneOS, LineageOS, /e/OS).

The Wallet Provider can optionally accept registrations that rely on the
hardware key attestation alone, without a Play Integrity token. This keeps a
hardware root of trust (it is a replacement of the integrity signal, not a
removal): the policy requires a hardware-backed attestation security level,
checks the verified boot state and bootloader lock state from the attestation's
root of trust, and binds the attestation to our app via the
`attestation_application_id` (package name and, optionally, signing certificate
digest). See `KeyAttestationOnlyPolicy` in
`service/src/account_server.rs`.

This path is **disabled by default**. Enable it in the settings under
`[android.key_attestation_only]`:

```toml
[android.key_attestation_only]
enabled = true
# "verified" is the strongest state (locked bootloader + verified OS, including
# relocked GrapheneOS). Accepting "self_signed"/"unverified" weakens the
# guarantee and should be a deliberate policy decision.
allowed_verified_boot_states = ["verified"]
require_device_locked = true
require_matching_signature_digest = true
```

When no token is present and this policy is not configured, the registration is
rejected, preserving the default behaviour.
