# Create a CA

To create a Certificate Authority (CA) for development purposes, you can use the
`wallet_ca` CLI tool included in the NL Wallet GitHub repository. We assume you
have followed the instructions in the `README.md` of the root of the NL Wallet
repository, so you have rust installed, run on a supported platform, etc.

In the following code block, we clone the nl-wallet repository, enter its
directory, set a target directory and specify an identifier (this identifier
resembles your organization, lowercase characters a-z, can end with numbers but
not begin with them).

We then make sure the target directory exists, and invoke `cargo` (rust's build
tool) to in turn invoke `wallet_ca` which creates the CA certificate and key.

Change at least the `IDENTIFIER` variable to match your organization name
(should be a value with lowercase characters a-z, and can end with numbers but
may not begin with them).

```shell
# Git clone and enter the nl-wallet repository if you haven't already done so.
git clone https://github.com/MinBZK/nl-wallet
cd nl-wallet

# Set and create target directory, identifier for your certificates.
export TARGET_DIR=target/ca-cert
export IDENTIFIER=foocorp
mkdir -p "${TARGET_DIR}"

# Create the CA certificate using wallet_ca.
cargo run --manifest-path "wallet_core/Cargo.toml" --bin "wallet_ca" ca \
    --common-name "ca.${IDENTIFIER}" \
    --file-prefix "${TARGET_DIR}/ca.${IDENTIFIER}"

# Convert certificate PEM to DER.
openssl x509 \
    -in "${TARGET_DIR}/ca.${IDENTIFIER}.crt.pem" -inform PEM \
    -out "${TARGET_DIR}/ca.${IDENTIFIER}.crt.der" -outform DER

# Convert key PEM to DER.
openssl pkcs8 -topk8 -nocrypt \
    -in "${TARGET_DIR}/ca.${IDENTIFIER}.key.pem" -inform PEM \
    -out "${TARGET_DIR}/ca.${IDENTIFIER}.key.der" -outform DER
```

After executing the above commands, you will have four files in the target
directory you specified: The CA certificate and key in both PEM and DER format.

<div class="admonition note">
<p class="title">Why create your own CA certificates?</p>
<p>Normally, when you set up a local development environment, the setup script
creates a couple of CAs for you (to be precise, one for Wallet Relying Party
Access Certificates (WRPACs), one for Wallet Relying Party Registration
Certificates (WRPRCs), one for issuers, and one for the Wallet Provider).</p>
<p>But there might be a reason for you to set up your own CA manually, for
example, when you [onboard][1] in our NL Wallet Community, you will need to
create your own CA and share the certificate of your CA with the operations
team.</p>
</div>

## Background

The participants in the [EDI-stelsel][2] can generally be expressed as the
following three entities:

  * Users (people that use the Wallet app)
  * Issuers (entities that issue attested attributes)
  * Verifiers (entities that want to verify a users' attested attributes)

To develop integrations within the EDI-stelsel (i.e., a verifier, or an issuer
that can interact with the app), you need to have certain certificates that
ascertain the validity of a verifier and issuer, and that contain various
specific attributes that make the certificate work with the NL Wallet.

These certificates are issued by a CA (Certificate Authority) and these CAs are
then trusted by the app, the issuers and the verifiers. Each type of
certificate is verified against its own list of trust anchors:

  * WRPACs (`wallet_ca cert --type wrpac`), which identify verifiers and
    issuers to the wallet, are verified against `wrpac_trust_anchors`.
  * WRPRCs (`--type wrprc`), which sign the registration certificate of a
    verifier or issuer, are verified against `wrprc_trust_anchors`. A WRPRC
    describes what an organization is registered for: the attributes a verifier
    may request, or the attestations an issuer provides.
  * Issuer certificates (`--type issuer`), which sign the attestations an issuer
    issues, are verified against `issuer_trust_anchors`.
  * Token Status List signing certificates (`--type tsl`) sign the status lists
    of attestations or WRPRCs, and are verified against the same trust anchors
    as the attestations or WRPRCs whose status they publish.

In the future, the EDI-stelsel will likely include a dedicated CA entity that
will be responsible for issuing certificates for both issuers and verifiers,
for both development and production purposes. This CA entity is not yet
available, and hence, we facilitate community participants by allowing them to
create and run their own CA.

## Publishing a CRL for WRPACs

A CA that issues WRPACs must publish a Certificate Revocation List (CRL), and
every WRPAC it issues must point to that CRL in its CRL distribution point. The
wallet rejects a WRPAC when its CRL is unavailable, invalid, expired, or lists
that certificate as revoked, so you need to replace the published CRL before its
`nextUpdate`. Creating, publishing and updating the CRL with `wallet_ca crl` is
explained in the guides to [create a verifier][3] and [create an issuer][4].

## Sharing your CA Certificate with the Operations Team

<div class="admonition note"><p class="title">Optional</p>
If you're creating your own CA because you are in the process of [onboarding][1]
in our NL Wallet Community, you need to share your CA certificate with the
operations team. This section covers that - if you're just creating your own CA
for usage in your own environment, please ignore this section.
</div>

After you have created the CA certificate and its key, we need to share the
certificate with the operations team. The certificate will be called something
like this:

    ca.foocorp.crt.pem

Where "foocorp" is the name you previously used as identifier. You can e-mail
the certificate to the e-mail address of the operations team. They will
configure this certificate in the issuer and reader trust anchors of the NL
Wallet. You will have obtained the e-mail address to be used after approval,
possibly at the kick-off.

Make sure you keep the key file safe, it should never be shared with any party
external to your organization (it's called something like `ca.foocorp.key.pem`).

[1]: /community/onboarding
[2]: /about
[3]: ../get-started/create-a-verifier.md#creating-a-wrpac-certificate
[4]: ../get-started/create-an-issuer.md#creating-issuer-tsl-wrpac-and-wrprc-certificates
