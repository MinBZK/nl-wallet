# PID Issuer

This folder contains a reference implementation for a PID issuer.

At the moment this application is set up for connecting to the digid-connector.

# Development environment

In order to test this issuer locally, and connect the wallet to it, follow the following steps:

1. Generate a 2048-bit RSA keypair (see below) and provision the digid-connect with the public key. Include the private key in JWK format in the settings.
3. Generate an issuer certificate and private key (see below), and include them in the settings.
4. Run the pid_issuer with `RUST_LOG=DEBUG cargo run --bin pid_issuer`.
5. Modify the `pid_issuer_url` variable in `wallet_core/wallet/src/config/data.rs` to "http://10.0.2.2:3003/" (for the android emulator) or "http://localhost:3003/" (for the iOS simulator).
6. Start the wallet.

## Settings
Default settings are specified in `src/settings.rs`. They are also shown in `pid_issuer.example.toml` (optional settings are commented out).

The default settings can be overridden in two ways:
- Create a file named `pid_issuer.toml` in the same location as `pid_issuer.example.toml`.
- Using environment variables. All environment variables should be prefixed with `PID_ISSUER`. Grouped settings can be specified as follows: `PID_ISSUER_WEBSERVER__PORT`, where the group name is separated from the key by a double underscore `__`. Environment variables take precedence over entries in `pid_issuer.toml`.

### Generating an RSA JWK keypair

In Rust, using the [`josekit` crate](https://docs.rs/josekit) a keypair can be generated as follows:

```rust
let keypair = josekit::jwe::RSA_OAEP.generate_key_pair(2048).expect("key generation failed");
let privkey = serde_json::to_string(&keypair.to_jwk_private_key()).unwrap();
let pubkey = serde_json::to_string(&keypair.to_jwk_public_key()).unwrap();
```

### Generating an issuer certificate and private key

The PID issuer needs an X509 certificate (which is included in the issued mdocs) and private key (with which it signs the mdocs). The certificate needs to be signed by a CA. If you do not already have a CA, you can generate it as follows (replacing the `CN` with a sensible value):

```sh
openssl req -x509 \
    -nodes -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 \
    -keyout ca_privkey.pem \
    -out ca_cert.pem \
    -days 365 \
    -addext keyUsage=keyCertSign,cRLSign \
    -subj '/CN=ca.example.com'
```

The CA certificate and CA private key are needed to generate the issuer certificate and issuer private key. The PID issuer itself does not need them to run. After the issuer certificate and private key have been generated, therefore, the CA certificate and private key should be moved to a secure location. Also the CA certificate must be added to the `TRUST_ANCHOR_CERTS` in `wallet_core/wallet/src/config/data.rs`.

Next, generate the issuer certificate and private key as follows.

```sh
openssl req -new \
    -nodes -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 \
    -keyout issuer_privkey.pem \
    -out issuer_csr.pem \
    -subj "/CN=pid.example.com" && \
openssl x509 -req \
    -extfile <(printf "keyUsage=digitalSignature\nextendedKeyUsage=1.0.18013.5.1.2\nbasicConstraints=CA:FALSE") \
    -in issuer_csr.pem \
    -days 365 \
    -CA ca_cert.pem \
    -CAkey ca_privkey.pem \
    -out issuer_cert.pem && \
rm issuer_csr.pem
```

Put the Base64-parts of these files, without newlines, in your configuration file or environment variables.
