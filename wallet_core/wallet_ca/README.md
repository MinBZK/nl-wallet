# Wallet CA

`wallet_ca` is a development and operations utility for creating the X.509
material used by NL Wallet. It can generate self-signed certificate authorities,
issuer and access certificates, certificates for existing public keys, signed
reader requests, OAuth Status List Tokens, and certificate revocation lists
(CRLs).

Run the CLI from the repository root:

```shell
cargo run --manifest-path wallet_core/Cargo.toml --bin wallet_ca -- --help
cargo run --manifest-path wallet_core/Cargo.toml --bin wallet_ca -- cert --help
```

## Registration certificates

A Wallet Relying Party Registration Certificate (WRPRC) is signed under the
WRPRC PKI, but its subject is bound to a separate Wallet Relying Party Access
Certificate (WRPAC). First create an end-entity signing key pair under the
WRPRC CA:

```shell
cargo run --manifest-path wallet_core/Cargo.toml --bin wallet_ca -- cert \
    --type wrprc \
    --ca-key-file target/ca.wrprc.key.pem \
    --ca-crt-file target/ca.wrprc.crt.pem \
    --common-name "Development WRPRC signer" \
    --organization-name "Development Registrar B.V." \
    --organization-id NTRNL-00000001 \
    --file-prefix target/wrprc-signer
```

Then sign an authored JSON payload. The command validates the payload and
checks that its `sub` matches the subject identifier in the supplied WRPAC
before signing it:

```shell
cargo run --manifest-path wallet_core/Cargo.toml --bin wallet_ca -- \
    registration-certificate \
    --wrprc-key-file target/wrprc-signer.key.pem \
    --wrprc-crt-file target/wrprc-signer.crt.pem \
    --wrpac-crt-file target/example-wrpac.crt.pem \
    --payload-file registration-certificate.json \
    --format cwt
```

Both `jwt` and `cwt` formats are supported. The command prints one unpadded
base64url string containing the serialized WRPRC, ready for a
`registration_certificate` configuration value or OpenID4VP
`verifier_info.data`. It does not create the payload, its referenced status
list, or deployment configuration.

## Status List Tokens

Create a separate Token Status List signing certificate under the same CA as
the referenced tokens. For WRPRCs, its subject distinguished name must be
identical to that of the WRPRC signing certificate. It remains a separate
certificate because status-list validation requires the OAuth Status Signing
extended key usage:

```shell
cargo run --manifest-path wallet_core/Cargo.toml --bin wallet_ca -- cert \
    --type tsl \
    --ca-key-file target/ca.wrprc.key.pem \
    --ca-crt-file target/ca.wrprc.crt.pem \
    --common-name "Development WRPRC signer" \
    --organization-name "Development Registrar B.V." \
    --organization-id NTRNL-00000001 \
    --file-prefix target/wrprc-tsl
```

Then generate the token, listing the status at every index in order. The
command prints the compact JWT, suitable for redirecting directly to the file
that will be published at the supplied URI:

```shell
cargo run --manifest-path wallet_core/Cargo.toml --bin wallet_ca -- \
    status-list \
    --tsl-key-file target/wrprc-tsl.key.pem \
    --tsl-crt-file target/wrprc-tsl.crt.pem \
    --uri https://example.com/wrprc/1 \
    --status valid valid valid \
    --valid-for-days 365 \
    --ttl-seconds 3600 \
    > target/wrprc-status-list.jwt
```

Supported entry values are `valid`, `revoked`, and `suspended`. Both
`--valid-for-days` and `--ttl-seconds` are optional; omitting them leaves the
corresponding `exp` and `ttl` claims out of the token.

## CRL distribution points

WRPAC consumers in the wallet require revocation checking. Generate a signed
CRL for the WRPAC CA:

```shell
cargo run --manifest-path wallet_core/Cargo.toml --bin wallet_ca -- crl \
    --ca-key-file target/ca-wrpac.key.pem \
    --ca-crt-file target/ca-wrpac.crt.pem \
    --file-prefix target/wrpac \
    --days 7
```

This creates `target/wrpac.crl.pem`. Convert it to DER for publication at a
stable HTTP or HTTPS URL, then embed that URL in every certificate issued by
the CA:

```shell
openssl crl -in target/wrpac.crl.pem -outform DER \
    -out target/wrpac.crl.der
```

```shell
cargo run --manifest-path wallet_core/Cargo.toml --bin wallet_ca -- cert \
    --type wrpac \
    --ca-key-file target/ca-wrpac.key.pem \
    --ca-crt-file target/ca-wrpac.crt.pem \
    --crl-distribution-point https://example.com/wrpac.crl.der \
    --common-name example \
    --organization-name "Example B.V." \
    --organization-id NTRNL-00000002 \
    --file-prefix target/example-wrpac
```

To revoke a certificate, obtain its serial number and regenerate the CRL with
that serial number included. Keep the previous PEM file at the same output
prefix so `wallet_ca` can advance its `crlNumber` even when the clock has not:

```shell
openssl x509 -in target/example-wrpac.crt.pem -noout -serial
cargo run --manifest-path wallet_core/Cargo.toml --bin wallet_ca -- crl \
    --ca-key-file target/ca-wrpac.key.pem \
    --ca-crt-file target/ca-wrpac.crt.pem \
    --file-prefix target/wrpac \
    --days 7 \
    --serial-number 0123456789abcdef \
    --force
```

Convert and republish the DER file before the previous CRL's `nextUpdate`.
The wallet rejects a WRPAC if it has no usable distribution point, the CRL
cannot be fetched or validated, or the certificate is listed as revoked. CRLs
are signed, so transport-level integrity is not required. ETSI EN 319 412-2,
`GEN-4.3.11-4`, requires at least one `http://` or `ldap://` CRL reference.
The wallet does not support LDAP retrieval, so include at least one `http://`
distribution point. Additional `https://` distribution points remain
supported.
