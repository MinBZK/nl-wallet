# Keycloak Setup

A local Keycloak for development purposes is configured by `setup-devenv.sh`
and started by `start-devenv.sh` (option `kc` / `keycloak`). The container
is a standard keycloak container (`quay.io/keycloak/keycloak:latest`), with
HTTPS terminated inside the container using a generated certificate (see the
`scripts/devenv/target/keycloak` directory for the generated certificates
and the `realm.json` file).

The realm is imported from a `realm.json`, copied into the `keycloak` volume
at `/opt/keycloak/data/import` using `docker compose run` (this is done like
this so it also works with remote `DOCKER_HOST` setups too). The realm is only
updated on a fresh volume, i.e., after editing `realm.json`, you need to drop
the volume and re-execute `setup-devenv.sh` which populates the volume.

In general, to run:

```bash
git clone nl-wallet
cd nl-wallet
scripts/setup-devenv.sh # configure the named volume
scripts/start-devenv.sh keycloak # starts the container
scripts/start-devenv.sh keycloak --stop # stops keycloak
```
## Login Credentials

Administration user: `keycloak` / `keycloak`, HTTP `:8080`, HTTPS `:8443`.
Overridable with: `KC_USERNAME`, `KC_PASSWORD`, `KC_PORT_HTTP`, `KC_PORT_HTTPS`.

## Realm Configuration

Realm name: `nl-wallet`
Realm file: `scripts/devenv/keycloak/realm.json`

### Privileges (realm roles)

| Role                        | Description                                                      |
| --------------------------- | ---------------------------------------------------------------- |
| `privilege_revoke_wallet`   | Create or review a task to revoke a Wallet.                      |
| `privilege_block_user`      | Create or review a task to block a user.                         |
| `privilege_unblock_user`    | Create or approve a task to unblock a blocked user.              |
| `privilege_show_all_tasks`  | See all open and completed tasks.                                |
| `privilege_revoke_solution` | Revoke the Wallet Solution.                                      |

Privileges are plain realm roles and appear in the access token as `realm_access.roles`.

### Test users

A couple of test users, password equals username.

| User           | Password        | Privileges                                                                   |
| --------------- | --------------- | --------------------------------------------------------------------------- |
| `administrator` | `administrator` | `privilege_revoke_wallet`, `privilege_block_user`, `privilege_unblock_user` |
| `manager`       | `manager`       | `privilege_show_all_tasks`                                                  |
| `revoker`       | `revoker`       | `privilege_revoke_solution`                                                 |

### Client

| Property     | Value                                         |
| ------------ | --------------------------------------------- |
| Client ID     | `wallet-backend`                             |
| Type          | Public (PKCE, no secret)                     |
| Flow          | `authorization_code` (`standardFlowEnabled`) |
| PKCE          | `S256` (`pkce.code.challenge.method`)        |
| Redirect URI  | `https://localhost:3000/auth/callback`       |
