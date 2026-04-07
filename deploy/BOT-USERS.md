## CI/CD Bot Users Variable Setup

### Prerequisites

- Access to the GitLab instance with rights to create Group Access Tokens
- GPG set up locally

Replace the following placeholder throughout:

| Placeholder    | Where to find it              |
| -------------- | ----------------------------- |
| `<GITLAB_URL>` | Your GitLab instance hostname |

---

### 1. Create a Project Access Token

Go to **Group → Settings → Access Tokens**, create a token named `<PREFIX>`
with **Developer** role and `api` (eventually `write_repository`) scope.
This is the `GROUP_ACCESS_TOKEN`. Copy the token value — it is only shown
once. Store it as CI/CD variable `<PREFIX>_GITLAB_TOKEN` (masked) via
**Group → Settings → CI/CD → Variables**.

---

### 2. Register your GPG public key with the bot user

Find the email address of the bot user created by the access token:

```bash
curl --silent --header "PRIVATE-TOKEN: <GROUP_ACCESS_TOKEN>" \
  "https://<GITLAB_URL>/api/v4/user" | jq '.email'
```

Generate a GPG key using that email address (no passphrase):

```bash
gpg --batch --gen-key <<EOF
%no-protection
Key-Type: EdDSA
Key-Curve: ed25519
Name-Real: <BOT_NAME>
Name-Email: <BOT_EMAIL>
Expire-Date: 0
EOF
```

Register the public key with the bot user:

```bash
curl --silent --request POST \
  --header "PRIVATE-TOKEN: <GROUP_ACCESS_TOKEN>" \
  --data-urlencode "key=$(gpg --armor --export <BOT_EMAIL>)" \
  "https://<GITLAB_URL>/api/v4/user/gpg_keys"
```

---

### 3. Set the remaining CI/CD variables

```bash
gpg --armor --export-secret-keys <BOT_EMAIL> | base64 -w 0
```

#### Renovate

Go to **Project → Settings → CI/CD → Variables** and add:

| Key                               | Value                       | Masked |
| --------------------------------- | --------------------------- | ------ |
| `RENOVATE_GIT_AUTHOR`             | `Renovate Bot <BOT_EMAIL>`  | No     |
| `RENOVATE_GPG_PRIVATE_KEY_BASE64` | Output of the command above | Yes    |

#### Audit Fix

Go to **Project → Settings → CI/CD → Variables** and add:

| Key                                | Value                       | Masked |
| ---------------------------------- | --------------------------- | ------ |
| `AUDIT_FIX_GIT_EMAIL`              | `<BOT_EMAIL>`.              | No     |
| `AUDIT_FIX_GPG_PRIVATE_KEY_BASE64` | Output of the command above | Yes    |
