# Keycloak Setup

A local Keycloak for development purposes is configured by `setup-devenv.sh`
and started by `start-devenv.sh` (option `kc` / `keycloak`). The container
is a standard keycloak container (`quay.io/keycloak/keycloak:latest`), with
HTTPS terminated inside the container using a generated certificate.

The directory `scripts/devenv/keycloak` mirrors the layout of the `keycloak`
named volume (mounted at `/opt/keycloak/data` in the container): it contains a
`certs/` directory with the generated TLS certificates (written once by
`setup-devenv.sh` and git-ignored) and an `import/` directory holding
`realm.json`, which Keycloak imports on container start.

`setup-devenv.sh` populates the named volume in a single pass by streaming a
self-extracting shell script through stdin to the keycloak container (using
`docker compose run`, so this also works with a remote `DOCKER_HOST`). The
keycloak image (Red Hat UBI 9 minimal) ships neither `tar` nor a package
manager, so each file is base64-encoded on the host and decoded in the
container with coreutils `base64`.

The realm is only imported on a fresh volume; after editing `realm.json`,
drop the volume and re-run `setup-devenv.sh` to repopulate it.

In general, to run:

```bash
git clone nl-wallet
cd nl-wallet
scripts/setup-devenv.sh # configure the named volume
scripts/start-devenv.sh keycloak # starts the container
scripts/start-devenv.sh keycloak --stop # stops keycloak
```
## Login Credentials

Administration user: `keycloak` / `keycloak`, HTTP `:11080`, HTTPS `:11443`.
Overridable with: `KC_USERNAME`, `KC_PASSWORD`, `KC_PORT_HTTP`, `KC_PORT_HTTPS`.

## Kubernetes (ont/demo)

The `keycloak` helm-chart in `deploy/helm-charts/keycloak` deploys the same
container to a Kubernetes namespace. Differences from local dev:

- TLS is terminated by the ingress/gateway; the pod serves plain HTTP on `:8080`
  (`KC_HTTP_ENABLED=true`, no in-pod certificates). The Service exposes `:443`
  → `:8080`.
- The whole `nl-wallet` realm import JSON (roles, users, passwords, clients)
  is supplied by an existing secret (`keycloak.realm.existingSecret`, default
  `nl-wallet-keycloak-realm`, key `realm.json`) and mounted read-only at
  `/opt/keycloak/data/import/realm.json`. The chart itself ships no realm
  contents and holds no realm secrets; creating the secret is a deploy-time
  concern handled out-of-band (separate repositories / pipelines for ont and
  demo). For a throw-away local cluster, the dev realm from
  `scripts/devenv/keycloak/import` can be used:

  ```shell
  kubectl create secret generic nl-wallet-keycloak-realm \
    --from-file=realm.json=scripts/devenv/keycloak/import/realm.json
  ```

- The admin user credentials come from an existing secret
  (`keycloak.admin.existingSecret`); set `KC_BOOTSTRAP_ADMIN_USERNAME` /
  `KC_BOOTSTRAP_ADMIN_PASSWORD` keys.
- State (`/opt/keycloak/data`) is on a PVC (if `persistence.enabled`). Defaults
  to the embedded `dev-file` database; switch `keycloak.database` to a real DB
  for `replicaCount > 1`.

> **Import runs once.** `--import-realm` only imports a realm that does not
> exist yet. With `persistence.enabled` the realm stays on the PVC, so later
> realm changes are not applied. Re-import requires wiping the realm or PVC.

Run `helm dependency build deploy/helm-charts/keycloak` before install (the
chart depends on `sp-common`).

## Realm Configuration

Realm name: `nl-wallet`
Realm file: `scripts/devenv/keycloak/import/realm.json`

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
| `destroyer`     | `destroyer`     | `privilege_revoke_solution`                                                 |

### Client

| Property     | Value                                         |
| ------------ | --------------------------------------------- |
| Client ID     | `wallet-backend`                             |
| Type          | Public (PKCE, no secret)                     |
| Flow          | `authorization_code` (`standardFlowEnabled`) |
| PKCE          | `S256` (`pkce.code.challenge.method`)        |
| Redirect URI  | `https://localhost:3000/auth/callback`       |
