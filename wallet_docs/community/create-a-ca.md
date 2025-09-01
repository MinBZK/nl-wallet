# Create a CA

To create a Certificate Authority (CA) for development purposes, you can use the
`wallet_ca` CLI tool included in the NL-Wallet GitHub repository. We assume you
have followed the instructions in the `README.md` of the root of the NL-Wallet
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
export TARGET_DIR=../ca-target
export IDENTIFIER=foocorp
mkdir -p "${TARGET_DIR}"

# Create the CA certificate using wallet_ca.
cargo run --manifest-path "wallet_core/Cargo.toml" --bin "wallet_ca" ca \
    --common-name "ca.${IDENTIFIER}" \
    --file-prefix "${TARGET_DIR}/ca.${IDENTIFIER}"
```

After executing the above commands, you will have two files in the target
directory you specified.

<div class="admonition note">
<p class="title">Why create your own CA certificates?</p>
<p>Normally, when you set up a local development environment, the setup script
creates a couple of CA's for you (to be precise, one for verifiers, and one for
issuers).</p>
<p>But there might be a reason for you to set up your own CA manually, for
example, when you [onboard][1] on our NL-Wallet community platform, you will
need to create your own CA and share the public key of your CA with the
operations team.</p>

## Background

The participants in the NL-Wallet ecosystem can generally be expressed as the
following three entities:

  * Users (people that use the Wallet app)
  * Issuers (entities that issue attested attributes)
  * Verifiers (entities that want to verify a users' attested attributes)

To develop integrations within the NL-Wallet ecosystem (i.e., a verifier, or an
issuer that can interact with the app), you need to have certain certificates
that ascertain the validity of a verifier and issuer, and that contain various
specific attributes that make the certicate work with our ecosystem.

These certificates are issued by a CA (Certificate Authority) and these CAs are
then trusted by the app, the issuers and the verifiers.

In the future, the NL-Wallet platform will likely facilitate or use a dedicated
CA entity that will be responsible for issuing certificates for both issuers and
verifiers, for both development and production purposes. This CA entity is not
yet available, and hence, we facilitate community participants by allowing them
to create and run their own CA.

## Sharing your CA Public Key with the Operations Team

<div class="admonition note"><p class="title">Optional</p>
If you're creating your own CA because you are in the process of [onboarding][1]
on our NL-Wallet community platform, you need to share your public key with
the operations team. This section covers that - if you're just creating your own
CA for usage in your own environment, please ignore this section.
</div>

After you have created the CA certificate and its key, we need to share the
certificate with the operations team. the certificate will be called something
like this:

    ca.foocorp.crt.pem

Where "foocorp" is the name you previously used as identifier. You can e-mail
the public key to the e-mail address of the operations team. They will configure
this key in the issuer and reader trust anchors of the NL-Wallet. You will have
obtained the e-mail address to be used after approval, possibly at the kick-off.

Make sure you keep the key file safe, it should never be shared with any party
external to your organization (it's called something like `ca.foocorp.key.pem`).

[1]: /community/onboarding
