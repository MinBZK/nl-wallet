# Setup development environment

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
