# Wallet CA

`wallet_ca` is a development and operations utility for creating the X.509
material used by NL Wallet. It can generate self-signed certificate authorities,
issuer and access certificates, certificates for existing public keys, signed
reader requests, and certificate revocation lists (CRLs).

Run the CLI from the repository root:

```shell
cargo run --manifest-path wallet_core/Cargo.toml --bin wallet_ca -- --help
cargo run --manifest-path wallet_core/Cargo.toml --bin wallet_ca -- cert --help
```

## CRL distribution points

WRPAC consumers in the wallet require revocation checking. Generate a signed
CRL for the WRPAC CA and publish the DER output at a stable HTTP or HTTPS URL:

```shell
cargo run --manifest-path wallet_core/Cargo.toml --bin wallet_ca -- crl \
    --ca-key-file target/ca-wrpac.key.pem \
    --ca-crt-file target/ca-wrpac.crt.pem \
    --file-prefix target/wrpac \
    --days 7
```

This creates `target/wrpac.crl.pem` for inspection and
`target/wrpac.crl.der` for publication. Embed the published URL in every
certificate issued by that CA:

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
that serial number included:

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

Republish the DER file before the previous CRL's `nextUpdate`. The wallet fails
closed if a WRPAC has no usable distribution point, the CRL cannot be fetched
or validated, or the certificate is listed as revoked. CRLs are signed, so an
HTTP distribution URL is supported, although HTTPS is preferable where it is
operationally convenient.
