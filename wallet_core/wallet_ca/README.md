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
