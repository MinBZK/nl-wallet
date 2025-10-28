# Create an Issuer

You want to create an issuer and issue attestations to NL Wallet users.

An issuer (also known as a "verstrekkende voorziening" in Dutch) is essentially
an entity that wants to issue attestations to the NL Wallet. An issuer issues
cards to the wallet which attest to certain facts about a person. Things like
diploma's, driving licenses, personalia, etc.

This document provides a global outline of components used, the necessary
decisions, data, and certificate(s), and guides the setup of a so-called
issuer/verstrekkende-voorziening.

<div class="admonition note"><p class="title">Open-source software</p>
Did you know that the NL Wallet platform is fully open-source? You can find
[the project on GitHub][1].
</div>

```{contents}
:backlinks: none
:depth: 5
:local:
```

## What we're going to cover

We'll start with a paragraph about the related architecture, with links to the
relevant architecture documents.

We'll then cover the creation of the technical attestation schema, the creation
of issuer and reader authentication documents and the corresponding reader and
issuer certificates, which are essential for identifying your service within
the NL Wallet ecosystem.

Next, we'll guide you through setting up the `issuance_server`. This includes
obtaining the software, configuring it (with an optional database backend), and
running it for the first time.

We'll also cover how to validate that your setup is working correctly, and
finally, we'll point you to the issuer API specifications.

## Architecture overview

Our issuance implementation adheres to the [OpenID4VCI][2] specification.

For PID issuance, we have a specialized issuer called `pid_issuer` which
interacts with [DigiD][3] through [RDO Max][4], and which can obtain citizen
data from [RViG's BRP][5]. This document does not cover PID issuance, although
many parts of this document do apply to PID issuance equally.

This document is about generic issuance, where the wallet can disclose certain
attributes to an issuer in order to obtain new, different attestations, hence
the name "disclosure-based issuance".

You can find the general project start architecture documents for the NL Wallet
on [Pleio][4] .These document the general platform architecture, solution
architecture, design considerations and global functional design use cases.

To get a clear view on what issuance (in both its specialized PID issuance, and
generic disclosure-based issuance form) looks like on the NL Wallet platform,
have a look at the following documents:

  * [Issuance with OpenID4VCI][6]
  * [Disclosure-based Iissuance][7]

### Platform components overview

The NL Wallet platform consists of:

  * **Issuers**: (also known as Verstrekkende Voorzieningen), which can issue
    attested attributes, and which this document is mainly about;
  * **Verifiers**: (also known as Ontvangende Voorzieningen or Relying Parties),
    which can verify attested attributes they are interested in;
  * **Backend**: services that run in the NL Wallet datacenter(s) or cloud that
    facilitate various functions for the mobile app (usually not interacted with
    directly, by either Issuers or Verifiers);
  * **App**: the NL Wallet mobile app, which contains attested attributes,
    received from Issuers, and which it can disclose to Verifiers.

Issuers configure and maintain an `issuance_server` on their own premises or
cloud environments, which they integrate with their own application, and which
interacts with the NL Wallet app, in order to issue attested attributes.

## Creating a technical attestation schema document

In the next two sub-sections we'll dive into what's needed to create a TAS
document, i.e., a "technical attestation schema". We'll first decide on the
required attributes for your TAS and then create an actual document. You're
free to change values mostly as you see fit, especially if you'll be running
this in a testing environment, but keep in mind that when this document is
intended for a production environment, more stringent rules might apply.

<div class="admonition note">
<p class="title">When you work on eventual production readiness</p>
If you plan to eventually bring your issuer into production readiness, you might
want to consider our [onboarding][11] process. When you are a member of the NL
Wallet community, you have access to community resources that can help with
validation of your TAS, `issuer_auth`, `reader_auth` and `issuance_server`
configuration files.
</div>

### Decide on required metadata for your TAS

The TAS contains the following particularly important data elements:

**ROOT**
| Key           | Type   | Description                                         |
| ------------- | ------ | --------------------------------------------------- |
| `vct`         | string | Verifiable credential type field                    |
| `name`        | string | Readable name of this verifiable credential         |
| `description` | string | Description of this verifiable credential           |
| `display`     | array  | Array of display objects, one per language          |
| `claims`      | array  | Array of claim objects                              |
| `schema`      | object | A v2020-12 JSON Schema object defining the claims   |

Claims have the following elements:

**CLAIMS**
| Key           | Type   | Description                                         |
| ------------- | ------ | --------------------------------------------------- |
| `path`        | array  | An array of strings describing the claim path       |
| `display`     | array  | Array of display objects, one per language          |
| `sd`          | string | Indicates whether a claim is selectively disclosable|
| `svg_id`      | string | Template identifier for the SVG rendering metadata  |


### Creating the technical attestation schema JSON document

Below you'll see an example of a TAS JSON document. You can use it as an example
for your own, or use it verbatim as-is to test disclosure-based issuance.

We'll assume the latter for now; in any case, later sections in this guide will
assume you'll save it as `target/is-config/insurance_metadata.json` within the
`nl-wallet` directory (i.e., where you cloned the git repository of the NL
Wallet). If you change the name or location, keep in mind that the various
script sections later on will have to be adjusted also).

Here is the example TAS for an insurance:

```json
{
  "vct": "com.example.insurance",
  "name": "Insurance credential",
  "description": "Insurance credential",
  "display": [
    {
      "lang": "en-US",
      "name": "Insurance",
      "description": "An insurance credential",
      "summary": "{{coverage}}",
      "rendering": {
        "simple": {
          "background_color": "#b2e1ea",
          "text_color": "#152a62"
        }
      }
    },
    {
      "lang": "nl-NL",
      "name": "Verzekering",
      "description": "Een verzekering attestatie",
      "summary": "{{coverage}}",
      "rendering": {
        "simple": {
          "background_color": "#b2e1ea",
          "text_color": "#152a62"
        }
      }
    }
  ],
  "claims": [
    {
      "path": ["product"],
      "display": [
        {
          "lang": "nl-NL",
          "label": "Product",
          "description": "Soort verzekering"
        },
        {
          "lang": "en-US",
          "label": "Product",
          "description": "Type of insurance"
        }
      ],
      "sd": "always"
    },
    {
      "path": ["coverage"],
      "display": [
        {
          "lang": "nl-NL",
          "label": "Dekking",
          "description": "Dekking van de verzekering"
        },
        {
          "lang": "en-US",
          "label": "Coverage",
          "description": "Coverage of the insurance"
        }
      ],
      "sd": "always",
      "svg_id": "coverage"
    },
    {
      "path": ["start_date"],
      "display": [
        {
          "lang": "nl-NL",
          "label": "Startdatum",
          "description": "Datum waarop de verzekering ingaat"
        },
        {
          "lang": "en-US",
          "label": "Start date",
          "description": "Date on which the insurance starts"
        }
      ],
      "sd": "always"
    },
    {
      "path": ["duration"],
      "display": [
        {
          "lang": "nl-NL",
          "label": "Duur",
          "description": "Duur van de verzekering"
        },
        {
          "lang": "en-US",
          "label": "Duration",
          "description": "Duration of the insurance"
        }
      ],
      "sd": "always"
    },
    {
      "path": ["customer_number"],
      "display": [
        {
          "lang": "nl-NL",
          "label": "Klantnummer",
          "description": "Klantnummer van de verzekerde"
        },
        {
          "lang": "en-US",
          "label": "Customer number",
          "description": "Customer number of the insured"
        }
      ],
      "sd": "always"
    }
  ],
  "schema": {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "title": "Insurance VCT Schema",
    "description": "The JSON schema that defines a insurance VCT",
    "type": "object",
    "properties": {
      "vct": {
        "type": "string"
      },
      "vct#integrity": {
        "type": "string"
      },
      "iss": {
        "type": "string"
      },
      "nbf": {
        "type": "number"
      },
      "exp": {
        "type": "number"
      },
      "cnf": {
        "type": "object"
      },
      "status": {
        "type": "object"
      },
      "sub": {
        "type": "string"
      },
      "iat": {
        "type": "number"
      },
      "attestation_qualification": {
        "type": "string"
      },
      "product": {
        "type": "string"
      },
      "coverage": {
        "type": "string"
      },
      "start_date": {
        "type": "string",
        "format": "date"
      },
      "duration": {
        "type": "string"
      },
      "customer_number": {
        "type": "string"
      }
    },
    "required": ["vct", "iss", "attestation_qualification", "product", "coverage"]
  }
}
```

You can modify the above, keeping obvious constraints in mind (have a look at
the [previous section](#decide-on-required-metadata-for-your-tas)). Modified or
not, make sure you save it somewhere, keeping our earlier warnings about file
name and location in mind.

## Creating an issuer authentication document

We're first going to create a so-called `issuer_auth` document.

The subsections below describe the decisions you need to make as an issuer with
regards to attributes you want to issue, what data we require from you, how to
create an issuer certificate for your disclosure-based issuance setup (which is
configured for usage within the `issuance_server` configuration).

In this guide, we assume you have [onboarded succesfully][11] - i.e., you are
running your own CA and the public key of that CA has been shared with the
operations team who will need to add your CA public key to the trust anchors of
the app.

<div class="admonition note"><p class="title">Onboarding optional</p>
Do note that onboarding is not strictly necessary - you *can* follow all steps
in this guide and observe things working in a local development environment -
but when you want to test your issuer with the NL Wallet platform (i.e., our
backend and mobile apps in our acceptance and pre-production environments), you
do need to be onboarded to get access to those environments.
</div>

### Decide on required metadata for your issuer_auth

An issuer certificate contains a bunch of metadata, which we store as a part
of the certificate in a so-called X.509v3 extension. We use this data to present
a view of you, the issuer, in the NL Wallet app GUI.

**ROOT**
| Key                             | Languages | Description                                                          |
| ------------------------------- | --------- | -------------------------------------------------------------------- |
| `organization.displayName`      | `nl+en`   | Name of the verifier as shown in the app app.                        |
| `organization.legalName`        | `nl+en`   | Legal name of the verifier.                                          |
| `organization.description`      | `nl+en`   | Short one-sentence description or mission statement of the verifier. |
| `organization.webUrl`           | -         | The home URL of the verifier.                                        |
| `organization.city`             | `nl+en`   | The home city of the verifier.                                       |
| `organization.category`         | `nl+en`   | Bank, Municipality, Trading, Delivery Service, etc.                  |
| `organization.logo.mimeType`    | -         | Logo mimetype, can be image/svg+xml, image/png or image/jpeg         |
| `organization.logo.imageData`   | -         | Logo image data. When SVG, an escaped XML string, else base64        |
| `organization.countryCode`      | -         | Two-letter country code of verifier residence.                       |
| `organization.kvk`              | -         | Chamber of commerce number of verifier.                              |
| `organization.privacyPolicyUrl` | -         | Link to verifier's privacy policy.                                   |

Note: In the `Languages` column where it says `nl+en` for example, please
provide both Dutch and English values.

### Creating the issuer_auth JSON document

When you've collected all the required metadata, you are ready to create the
`issuer_auth.json` file. Here is an example for our insurance company:


```json
{
  "organization": {
    "displayName": {
      "nl": "VerzekerAar",
      "en": "InsurAnce"
    },
    "legalName": {
      "nl": "VerzekerAar N.V.",
      "en": "VerzekerAar N.V."
    },
    "description": {
      "nl": "VerzekerAar is een voorbeeld-verzekeraar.",
      "en": "InsurAnce is an exemplar insurance company."
    },
    "webUrl": "https://insurance.example.com",
    "city": {
      "nl": "Den Haag",
      "en": "The Hague"
    },
    "category": {
      "nl": "Verzekeringen",
      "en": "Insurance"
    },
    "logo": {
      "mimeType": "image/svg+xml",
      "imageData": "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"64\" height=\"64\" fill=\"none\"><rect width=\"64\" height=\"64\" y=\"-.002\" fill=\"#3A839A\" rx=\"12\"/><path fill=\"#FCFCFC\" d=\"M29.563 33.6H25.5v-4.8h4.063v-4h4.875v4H38.5v4.8h-4.062v4h-4.876zM32 16l-13 4.8v9.744C19 38.624 24.541 46.16 32 48c7.459-1.84 13-9.376 13-17.456V20.8zm9.75 14.544c0 6.4-4.144 12.32-9.75 14.128-5.606-1.808-9.75-7.712-9.75-14.128v-7.52l9.75-3.6 9.75 3.6z\"/></svg>"
    },
    "countryCode": "nl",
    "kvk": "99876543",
    "privacyPolicyUrl": "https://insurance.example.com/privacy"
  }
}
```

Take the above example, make sure you've read the previous sections which
explain what the different key/values mean, and (optionally) construct your
own `issuer_auth.json` file (or copy it verbatim if you're just testing). When
we are going to be creating the issuer certificate in the next sections, we are
going to need it at a specific location, so save it (inside the `nl-wallet`
git directory):

```
target/is-config/issuer_auth.json
```

## Creating a reader authentication document

We're going to create a so-called `reader_auth` document.

The subsections below describe the decisions you need to make as a verifier with
regards to attributes you want to verify, what data we require from you, how to
create a reader certificate for your usecase (which is configured for usage
within the `verification_server` configuration).

In this guide, we assume you have [onboarded succesfully][11] - i.e., you are
running your own CA and the public key of that CA has been shared with the
operations team who will need to add your CA public key to the trust anchors of
the app.

<div class="admonition note"><p class="title">Onboarding optional</p>
Do note that onboarding is not strictly necessary - you *can* follow all steps
in this guide and observe things working in a local development environment -
but when you want to test your verifier with the NL Wallet platform (i.e., our
backend and mobile apps in our acceptance and pre-production environments), you
do need to be onboarded to get access to those environments.
</div>

<div class="admonition note">
<p class="title">This chapter is also a part of creating a verifier</p>
Note that when you've also [created a verifier][31], this section will look
familiar to you; that is because an issuer, like a verifier, needs a reader
authentication document. This is because of how disclosure-based-issuance works:
with disclosure-based-issuance, an issuer is essentially also a verifier (i.e.,
you disclose some attributes in order to obtain some new ones).
</div>

### Decide on required metadata for your reader_auth

A reader certificate contains a bunch of metadata, which we store as a part
of the certificate in a so-called X.509v3 extension. We use this data to know
which attested attribute you want to verify, and to present a view of you, the
verifier in the NL Wallet app GUI.

**ROOT**
| Key                             | Languages | Description                                                          |
| ------------------------------- | --------- | -------------------------------------------------------------------- |
| `purposeStatement`              | `nl+en`   | For what purpose are you attesting? Login? Age verification? etc.    |
| `retentionPolicy`               | -         | Do you have an intent to retain data? For how long?                  |
| `sharingPolicy`                 | -         | Do you have an intent to share data? With whom?                      |
| `deletionPolicy`                | -         | Do you allow users to request deletion of their data, yes/no?        |
| `organization.displayName`      | `nl+en`   | Name of the verifier as shown in the app app.                        |
| `organization.legalName`        | `nl+en`   | Legal name of the verifier.                                          |
| `organization.description`      | `nl+en`   | Short one-sentence description or mission statement of the verifier. |
| `organization.webUrl`           | -         | The home URL of the verifier.                                        |
| `organization.city`             | `nl+en`   | The home city of the verifier.                                       |
| `organization.category`         | `nl+en`   | Bank, Municipality, Trading, Delivery Service, etc.                  |
| `organization.logo.mimeType`    | -         | Logo mimetype, can be image/svg+xml, image/png or image/jpeg         |
| `organization.logo.imageData`   | -         | Logo image data. When SVG, an escaped XML string, else base64        |
| `organization.countryCode`      | -         | Two-letter country code of verifier residence.                       |
| `organization.kvk`              | -         | Chamber of commerce number of verifier.                              |
| `organization.privacyPolicyUrl` | -         | Link to verifier's privacy policy.                                   |
| `authorizedAttributes`          | -         | List of attributes you want to verify.                               |

Note: In the `Languages` column where it says `nl+en` for example, please
provide both Dutch and English values.

### Decide on attributes you want to verify

You can verify any attribute provided by any issuer on the plaform, but since
we don't have an issuer registry yet, you would need to know or otherwise get
your hands on the JSON documents that define the claim paths that belong to a
given `vct` (a Verifiable Credential Type).

For our own issuer(s), you can use the `jq` utility to query our supported
attribute names:

```shell
git clone https://github.com/MinBZK/nl-wallet
cd nl-wallet/wallet_core/lib/sd_jwt_vc_metadata/examples
jq -r '(select(.vct | startswith("urn:")) | .vct) + ": " + (.claims[].path | join("."))' *.json | sort -u
```

The above `jq` command will output a sorted unique list of namespaces and the
attribute name that namespace supports. You will need one or more of those to
configure the `authorizedAttributes` object in `reader_auth.json`.

For example, suppose you want to verify `age_over_18` and `address.country`,
then your `authorizedAttributes` object would look as follows:

```json
"authorizedAttributes": {
  "urn:eudi:pid:nl:1": [["urn:eudi:pid:nl:1", "age_over_18"]],
  "urn:eudi:pid-address:nl:1": [["urn:eudi:pid-address:nl:1", "address.country"]],
}
```

In the case of our example insurance company, we're interested in the `bsn`, so
in the next section, where we show an example `reader_auth.json` document, we
simply use the following:

```json
"authorizedAttributes": {
  "urn:eudi:pid:nl:1": [
    ["urn:eudi:pid:nl:1", "bsn"]
  ]
```

### Creating the reader_auth JSON document

When you've collected all the required metadata, you are ready to create the
`reader_auth.json` file. Here is an example for our insurance company:

```json
{
  "purposeStatement": {
    "nl": "Uitgifte",
    "en": "Issuance"
  },
  "retentionPolicy": {
    "intentToRetain": true,
    "maxDurationInMinutes": 525600
  },
  "sharingPolicy": {
    "intentToShare": false
  },
  "deletionPolicy": {
    "deleteable": false
  },
  "organization": {
    "displayName": {
      "nl": "VerzekerAar",
      "en": "InsurAnce"
    },
    "legalName": {
      "nl": "VerzekerAargh N.V.",
      "en": "VerzekerAargh N.V."
    },
    "description": {
      "nl": "VerzekerAargh is een voorbeeld-verzekeraar.",
      "en": "InsurAnce is an exemplar insurance company."
    },
    "webUrl": "https://insurance.example.com",
    "city": {
      "nl": "Den Haag",
      "en": "The Hague"
    },
    "category": {
      "nl": "Verzekeringen",
      "en": "Insurance"
    },
    "logo": {
      "mimeType": "image/svg+xml",
      "imageData": "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"64\" height=\"64\" fill=\"none\"><rect width=\"64\" height=\"64\" y=\"-.002\" fill=\"#3A839A\" rx=\"12\"/><path fill=\"#FCFCFC\" d=\"M29.563 33.6H25.5v-4.8h4.063v-4h4.875v4H38.5v4.8h-4.062v4h-4.876zM32 16l-13 4.8v9.744C19 38.624 24.541 46.16 32 48c7.459-1.84 13-9.376 13-17.456V20.8zm9.75 14.544c0 6.4-4.144 12.32-9.75 14.128-5.606-1.808-9.75-7.712-9.75-14.128v-7.52l9.75-3.6 9.75 3.6z\"/></svg>"
    },
    "countryCode": "nl",
    "kvk": "99876543",
    "privacyPolicyUrl": "https://insurance.example.com/privacy"
  },
  "requestOriginBaseUrl": "https://insurance.example.com",
  "authorizedAttributes": {
    "urn:eudi:pid:nl:1": [
      ["urn:eudi:pid:nl:1", "bsn"],
      ["bsn"]
    ]
  }
}
```

Take the above example, make sure you've read the previous sections which
explain what the different key/values mean, and (optionally) construct your
own `reader_auth.json` file (or copy it verbatim if you're just testing). When
we are going to be creating the reader certificate in the next sections, we are
going to need it at a specific location, so save it (inside the `nl-wallet`
git directory):

```
target/is-config/reader_auth.json
```

## Creating issuer, reader and tsl certificates

Let's create the issuer, reader and tsl certificates. We're going to clone the
NL Wallet repository, enter its directory, set a target directory and specify an
identifier (this identifies your organization, and should be in lowercase
characters a-z, can end with numbers but may not begin with them).

We then make sure the target directory exists, and invoke `cargo` (rust's build
tool) to in turn invoke `wallet_ca` which will create the issuer, reader and tsl
certificates and keys.

Finally, we invoke `openssl` to convert our PEM certificates and key into DER
format.

<div class="admonition caution">
<p class="title">Do you have a working toolchain?</p>
Make sure you have a working toolchain as documented in our GitHub project root
`README.md` [here][19]. Specifically, you need to have `rust` and `openssl`
installed and working.
</div>

<div class="admonition caution">
<p class="title">Did you create an issuer_auth.json and reader_auth.json?</p>
You need valid `issuer_auth.json` and `reader_auth.json`, documents, which you
should have, if you followed along with the previous sections where we created
[issuer](#creating-the-issuer_auth-json-document) and
[reader](#creating-the-reader_auth-json-document) authorization documents.
</div>

<div class="admonition caution">
<p class="title">Did you create your own CA?</p>
You need a CA certificate and key. By default, when you're running locally, the
`setup-devenv.sh` script will have created these for you. You can also opt to
create your own custom self-signed CA certificate and key, which is documented
in the [Create a CA][27] document, and which is required if you need to
participate in the NL Wallet community platform.
</div>

<div class="admonition caution">
<p class="title">Do you intend to test your issuer on the NL Wallet platform?</p>
You can test your issuer locally (more or less exactly like we do with our
`demo-issuer` app) for which you don't need anything except the code in
our git repository. But if you want to test your issuer with the NL Wallet
platform (i.e., the NL Wallet apps on our Test Flight and Play Store Beta
environments plus backends), you will need to have succesfully completed the
[onboarding][11] process.
</div>

```shell
# Git clone and enter the nl-wallet repository if you haven't already done so.
git clone https://github.com/MinBZK/nl-wallet
cd nl-wallet

# Set and create target directory, identifier for your certificates.
export CA_DIR=target/ca-cert
export TARGET_DIR=target/vs-config
export IDENTIFIER=foocorp
mkdir -p "${CA_DIR}" "${TARGET_DIR}"

# Create the issuer certificate using wallet_ca.
cargo run --manifest-path "wallet_core/Cargo.toml" --bin "wallet_ca" issuer \
    --ca-key-file "${CA_DIR}/ca.${IDENTIFIER}.key.pem" \
    --ca-crt-file "${CA_DIR}/ca.${IDENTIFIER}.crt.pem" \
    --common-name "issuer.${IDENTIFIER}" \
    --issuer-auth-file "${TARGET_DIR}/issuer_auth.json" \
    --file-prefix "${TARGET_DIR}/issuer.${IDENTIFIER}"

# Create the reader certificate using wallet_ca.
cargo run --manifest-path "wallet_core/Cargo.toml" --bin "wallet_ca" reader \
    --ca-key-file "${CA_DIR}/ca.${IDENTIFIER}.key.pem" \
    --ca-crt-file "${CA_DIR}/ca.${IDENTIFIER}.crt.pem" \
    --common-name "reader.${IDENTIFIER}" \
    --reader-auth-file "${TARGET_DIR}/reader_auth.json" \
    --file-prefix "${TARGET_DIR}/reader.${IDENTIFIER}"

# Create the tsl certificate using wallet_ca.
cargo run --manifest-path "wallet_core/Cargo.toml" --bin "wallet_ca" tsl \
        --ca-key-file "${CA_DIR}/ca.issuer.key.pem" \
        --ca-crt-file "${CA_DIR}/ca.issuer.crt.pem" \
        --common-name "tsl.${IDENTIFIER}" \
        --file-prefix "${TARGET_DIR}/tsl.${IDENTIFIER}"

# Convert certificates PEM to DER.
openssl x509 \
    -in "${TARGET_DIR}/issuer.${IDENTIFIER}.crt.pem" -inform PEM \
    -out "${TARGET_DIR}/issuer.${IDENTIFIER}.crt.der" -outform DER
openssl x509 \
    -in "${TARGET_DIR}/reader.${IDENTIFIER}.crt.pem" -inform PEM \
    -out "${TARGET_DIR}/reader.${IDENTIFIER}.crt.der" -outform DER
openssl x509 \
    -in "${TARGET_DIR}/tsl.${IDENTIFIER}.crt.pem" -inform PEM \
    -out "${TARGET_DIR}/tsl.${IDENTIFIER}.crt.der" -outform DER

# Convert keys PEM to DER.
openssl pkcs8 -topk8 -nocrypt \
    -in "${TARGET_DIR}/issuer.${IDENTIFIER}.key.pem" -inform PEM \
    -out "${TARGET_DIR}/issuer.${IDENTIFIER}.key.der" -outform DER
openssl pkcs8 -topk8 -nocrypt \
    -in "${TARGET_DIR}/reader.${IDENTIFIER}.key.pem" -inform PEM \
    -out "${TARGET_DIR}/reader.${IDENTIFIER}.key.der" -outform DER
openssl pkcs8 -topk8 -nocrypt \
    -in "${TARGET_DIR}/tsl.${IDENTIFIER}.key.pem" -inform PEM \
    -out "${TARGET_DIR}/tsl.${IDENTIFIER}.key.der" -outform DER
```

The used CA public certificate (referenced in the previous `wallet_ca` command)
needs to be in the list of various so-called trust anchors. Specifically,
issuers and verifiers, and the NL Wallet app itself need to know if this CA is
a trusted CA, and our software "knows" that by checking its trust anchors.

When you run locally, when using `setup-devenv.sh` and `start-devenv.sh`, the
generated CA certificate is automatically added to the trust anchors within the
configuration files of `pid_issuer`, `demo_issuer`, `verification_server`,
`issuance_server`, `demo_relying_party` and the NL Wallet app config.

When you [create your own CA][27], you need to make sure the public key of your
CA is in the relevant trust anchor configuration settings. When you are a
member of the [NL Wallet community][11], and so using NL Wallet managed backend
services and mobile apps, this is done for you (i.e., you just need to sign
your issuer certificate with your CA, which the `wallet_ca` utility invocation
above did for you, and during the NL Wallet community [onboarding][11] process
you shared your CA certificate with the operations team who ensure your CA is
in the various trust anchor lists).

When you run locally, but with a manually created CA, you need to add the CA
public certificate to your services and wallet app config yourself. When we
generate the configuration later in this guide, we will do this automatically,
provided you used the naming conventions we used in the previous `wallet_ca`
invocations.

## Issuance server setup

In the following sections, we'll guide you through obtaining the software,
setting up a database backend and creating the `issuance_server` configuration
file.

### Obtaining the software

The `issuance_server` binary can be obtained by downloading a pre-compiled
binary from our [releases][20] page, or by compiling from source. To compile
from source, make sure you have our git repository checked out and make sure
you've [configured your local development environment][19]. Then:

```shell
cd nl-wallet
cargo build \
  --manifest-path wallet_core/Cargo.toml \
  --package issuance_server \
  --bin issuance_server \
  --locked --release
```

The above command creates `wallet_core/target/release/issuance_server`, which is
a release binary for the platform you're running on. Let's copy that binary to
our target config directory for usage later:

```shell
mkdir -p target/is-config
cp wallet_core/target/release/issuance_server target/is-config
```

<div class="admonition note">
<p class="title">About default feature flags</p>
Note that since we don't specify a `--features` argument in the above `cargo`
command, the default feature flags apply. For `issuance_server`, this
happens to be just `postgres`. When you build for local development, the build
script enables another feature flag called `allow_insecure_url`, which would
allow you to talk to an `issuance_server` using an (insecure) `http://` URL.
</div>

<div class="admonition danger">
<p class="title">Don't allow insecure URLs on production-like environments</p>
Don't enable `allow_insecure_url` on anything remotely production-like. Doing
so anyway, accidentally or not, could expose you to man-in-the-middle attacks.
</div>

### Using a database backend (optional)

The `issuance_server`, when compiled with the `postgres` feature flag
(which is the default), can use a PostgreSQL database backend to store state.
This is done by configuring a `postgres://` storage URL in the
`issuance_server.toml` configuration file. In this section, we'll create a
PostgreSQL database, configure credentials and configure the schema
(tables, columns).

<div class="admonition tip">
<p class="title">You can also run without a database backend</p>
Note that you can run `issuance_server` with a storage URL `memory://`
This makes the server store session state in memory. **When using in-memory
session state, on server shutdown or crash, any session state will be lost.**
</div>

#### Setting up PostgreSQL

You might already have a PostgreSQL database running, in which case you need the
credentials of a database user with `createdb` and `createrole` role attributes,
and the hostname of the system running the PostgreSQL database (can be localhost
or any fully-qualified domain name).

When you don't have a PostgreSQL database service running, you can create one
following the [installation instructions][22] or you can use something like
[docker][23] to run a containerized PostgreSQL service, which we'll document
here.

<div class="admonition note">
<p class="title">Use correct credentials and hostname in commands below</p>
When you decide to use your own previously configured PostgreSQL database
service, make sure you don't execute the `docker run` command which creates
a new PostgreSQL database service, and make sure you use the correct hostname,
username an password values.
</div>

```shell
# Specify database hostname, superuser name and password for PostgreSQL itself
# (change these if you're using you own previously created database service):
export PGHOST=localhost
export PGPORT=5432
export PGUSER=postgres
export PGPASSWORD="$(openssl rand -base64 12)"

# Specify database hostname, issuance_server database name, user name and
# password for verification_server:
export DB_NAME=issuance_server
export DB_USERNAME=wallet
export DB_PASSWORD="$(openssl rand -base64 12)"
```

Let's use Docker to run PostgreSQL, using a volume named `postgres` for the
database storage. We'll run in the background (the `--detach` option) and
auto-clean up the running container after stop (`--rm`). We're using the
previously declared environment variables for hostname, user and password
values:

```shell
# Run a Docker image named postgres, set superuser password to $PGPASSWORD.
docker run --name postgres --volume postgres:/var/lib/postgresql/data \
--rm  --detach --publish $PGPORT:5432 --env POSTGRES_PASSWORD="$PGPASSWORD" postgres
```

The next sections will use the environment variables declared previously (and
whichever database they point to).

#### Create user and database

Next, we'll create a user for the database and the database itself:

```shell
# Below psql commands use PGHOST, PGPORT, PGUSER and PGPASSWORD to execute.
psql -c "create user $DB_USERNAME with password '$DB_PASSWORD';"
psql -c "create database $DB_NAME owner $DB_USERNAME;"
```

#### Apply database schema

To initialize the `issuance_server` database schema, we will utilize the
migration tool helper:

<div class="admonition danger">
<p class="title">Applying the database schema using fresh is destructive</p>
Applying the `issuance_server` database schema using the `fresh` argument is
destructive! Any tables are cleared, data will be destroyed. Be sure you don't
run this on a currently operational production copy of `issuance_server`.
The migration tool also supports an ostensibly non-destructive argument `up`
which would not re-initialize the entire database, but as of this writing
(2025-10-26) we don't yet guarantee that our database initialization scripts
are non-changing, and hence, `up` might not work as intended.
</div>

```shell
cd nl-wallet
DATABASE_URL="postgres://$DB_USERNAME:$DB_PASSWORD@$PGHOST:$PGPORT/$DB_NAME" \
cargo run \
  --manifest-path wallet_core/wallet_server/issuance_server/Cargo.toml \
  --package issuance_server_migrations \
  --bin issuance_server_migrations -- fresh
```

You can show the configuration by issuing the following (might be a good idea
to keep this safe somewhere):

```shell
echo -e "\npostgres.host: '$PGHOST'\npostgres.port: '$PGPORT'\npostgres.user: '$PGUSER'\npostgres.pass: '$PGPASSWORD'\ndatabase.name: '$DB_NAME'\ndatabase.user: '$DB_USERNAME'\ndatabase.pass: '$DB_PASSWORD'\n"
```

### Creating a configuration

In the following sections we'll create the parts which will make up the
`issuance_server.toml` configuration file. Sections marked as "(optional)" can
be skipped. In the [final section]((#writing-the-configuration-file) we assemble
the actual `issuance_server.toml` file.

<div class="admonition note">
<p class="title">Example configuration file</p>
For reference, we have an annotated [example configuration file][24] which you
can check for the various settings you can configure. We cover most (all?) of
them here.
</div>

<div class="admonition note">
<p class="title">Similarity to verification_server</p>
If you've also [created a verifier][9], this part of the documentation will look
familiar, if slightly different here and there. This is in part because the
configuration of an `issuance_server` relies for a significant part on the
shared `wallet_server` code, which a `verification_server` also relies upon.
</div>

#### Logging settings (optional)

To configure request logging and specify if we want the log output in the JSON
format, we set the following:

```shell
cd nl-wallet
export TARGET_DIR=target/is-config && mkdir -p "$TARGET_DIR/parts"
cat <<EOF > "$TARGET_DIR/parts/01-logging-settings.toml"
log_requests = true
structured_logging = false
EOF
```

<div class="admonition note">
<p class="title">Optional runtime logging using env_logger</p>
In addition to the above, the NL Wallet uses [env_logger][17], which means you
can use the `RUST_LOG` environment variable when running `issuance_server`
later on. For example, to run with debug log output, you can prefix the command
with the `RUST_LOG` environment variable: `RUST_LOG=debug ./issuance_server`
</div>

#### Configuring trust anchors

[When you created the issuer, reader and tsl certificates](#creating-issuer-reader-and-tsl-certificates),
you signed those certificates using a CA, either generated by the development
setup script or specifically [created by you][27] as part of the (optional)
[community onboarding process][11].

The `issuance_server` distinguishes two kinds of trust anchors:

  * `issuer_trust_anchors` - a string array of CA certificates which are
    considered trusted to sign issuer certificates, in DER format, base64
    encoded;
  * `reader_trust_anchors` - a string array of CA certificates which are
    considered trusted to sign reader certificates, in DER format, base64
    encoded;

The trust anchor arrays tell the `issuance_server` which certificates it can
trust. If an `issuance_server` is presented with certificates signed by a CA
that is not in its trust anchor arrays, operations will fail (by design).

We need to trust our own CA, whether it is created by the development setup
scripts or explicitly by you. The development scripts create a separate CA for
issuers and readers (usually at `scripts/devenv/target/ca.issuer.crt.der` and
`scripts/devenv/target/ca.reader.crt.der`). When you create and use your own
CA for community development purposes as [documented here][27], you can use that
CA generally for signing both issuance and reader certificates, and hence, add
it to both the issuer and reader trust anchors.

The below code block will initialize the issuer and reader trust anchor
environment variables with the CA certificates it can find, both generated by
development scripts and any you created yourself, provided you [followed the CA
creation instructions to the letter][27] and used the naming convention
documented there which means you would have a `target/ca-cert` directory with
your CA certificates in DER format there. The code block assumes you have the
`nl-wallet` git repository checked out.

```shell
cd nl-wallet
export IS_ISSUER_TRUST_ANCHORS=()
export IS_READER_TRUST_ANCHORS=()
for i in scripts/devenv/target/ca.issuer.crt.der target/ca-cert/ca.*.crt.der; do \
    [[ -f $i ]] && IS_ISSUER_TRUST_ANCHORS+=($(openssl base64 -e -A -in $i)); done
for r in scripts/devenv/target/ca.reader.crt.der target/ca-cert/ca.*.crt.der; do \
    [[ -f $r ]] && IS_READER_TRUST_ANCHORS+=($(openssl base64 -e -A -in $r)); done

export TARGET_DIR=target/is-config && mkdir -p "$TARGET_DIR/parts"
cat <<EOF > "$TARGET_DIR/parts/03-trust-anchors.toml"
issuer_trust_anchors = [$(printf '"%s",' "${IS_ISSUER_TRUST_ANCHORS[@]}" | sed 's/,$//')]
reader_trust_anchors = [$(printf '"%s",' "${IS_READER_TRUST_ANCHORS[@]}" | sed 's/,$//')]
EOF
unset IS_ISSUER_TRUST_ANCHORS IS_READER_TRUST_ANCHORS
```

#### Determine public URL

The `public_url` is the URL that is used by the NL Wallet app to reach the
address and port of the `issuance_server`:

```shell
cd nl-wallet
export TARGET_DIR=target/is-config && mkdir -p "$TARGET_DIR/parts"
cat <<EOF > "$TARGET_DIR/parts/04-public-url.toml"
public_url = "https://issuer.example.com/"
EOF
```

<div class="admonition note">
<p class="title">Use a valid domain name here</p>
In the above, we use `issuer.example.com` as the fully-qualified domain name.
Technically, this domain needs not be world-reachable, but it does need to DNS
resolve for the NL Wallet app and the `issuance_server`. Make sure you use
a domain that is yours and that you control.
</div>

<div class="admonition warning">
<p class="title">A note about allowed public URL schemes</p>
When you [built or otherwise obtained](#obtaining-the-software) the software,
you did **not** specify the `allow_insecure_url` feature flag. This means you
cannot specify an `http://` URL here, and *need* to specify an `https://` URL.
</div>

#### Universal link base URL

The `issuance_server` uses the universal link base URL to construct the
correct environment-specific universal link. A universal link is used to to
associate a specific domain name and/or part of an URL with a specific app on
the mobile device. In our case, it results in the link provided by the
`issuance_server` being handled by the NL Wallet app when a user clicks
on the link or scans the QR code.

A universal link base URL is usually associated with a specific backend
environment like pre-production or testing. When you're integrating with the
NL Wallet platform, you would use a universal link base URL that was provided
to you as part of our community [onboarding][11] process.

```shell
cd nl-wallet
export TARGET_DIR=target/is-config && mkdir -p "$TARGET_DIR/parts"
cat <<EOF > "$TARGET_DIR/parts/05-universal-link-base-url.toml"
universal_link_base_url = "https://app.example.com/ul/"
EOF
```

<div class="admonition warning">
<p class="title">Make sure your domain is configured correctly</p>
You as the owner of the domain (`example.com` in the above example setting)
need to make sure the domain is configured correctly for universal links to
work correctly. On Apple iOS devices this is done with [associated domains][25].
On Google Android this is configured using [app links][26].
</div>

#### Configuring allowed client IDs

You can restrict which NL Wallet apps are accepted by the `issuance_server` by
configuring a `wallet_client_ids` array. The entries of this array would
contain the `client_id` value of a wallet implementation. This allows you to
allow-list groups of wallet apps based on their `client_id` value. For example,
for allowing apps that have `https://wallet.edi.rijksoverheid.nl` configured as
`client_id`:

```shell
cd nl-wallet
export IS_WALLET_CLIENT_IDS=("https://wallet.edi.rijksoverheid.nl")
export TARGET_DIR=target/is-config && mkdir -p "$TARGET_DIR/parts"
cat <<EOF > "$TARGET_DIR/parts/06-wallet-client-ids.toml"
wallet_client_ids = [$(printf '"%s",' "${IS_WALLET_CLIENT_IDS[@]}" | sed 's/,$//')]
EOF
unset IS_WALLET_CLIENT_IDS
```

#### Configuring metadata document references

We [previously](#creating-the-technical-attestation-schema-json-document) made
a technical attestation schema JSON document. The `issuance_server` needs to
know about these schemas. We can tell the server about available schemas through
the `metadata` setting. In this section, we're going to reference the previously
created JSON document `insurance_metadata.json`, which, if you followed the
instructions, was copied to `target/is-config` within the `nl-wallet` directory,
where the `issuance_server` will find it using the below configuration (provided
it is started from the `target/is-config` directory):


```shell
cd nl-wallet
export IS_WALLET_METADATA_FILES=("insurance_metadata.json")
export TARGET_DIR=target/is-config && mkdir -p "$TARGET_DIR/parts"
cat <<EOF > "$TARGET_DIR/parts/08-wallet-metadata-files.toml"
metadata = [$(printf '"%s",' "${IS_WALLET_METADATA_FILES[@]}" | sed 's/,$//')]
EOF
unset IS_WALLET_METADATA_FILES
```

#### Configuring listener address and port

The server can be configured to listen on a single IP address and port. The
address needs to be reachable by the NL Wallet mobile app:

```shell
cd nl-wallet
export TARGET_DIR=target/is-config && mkdir -p "$TARGET_DIR/parts"
cat <<EOF > "$TARGET_DIR/parts/09-listener-addresses-and-ports.toml"

[wallet_server]
ip = "0.0.0.0"
port = 8001
EOF
```

<div class="admonition warning">
<p class="title">Configure a correct IP address</p>
In the above configuration settings, we set `0.0.0.0` as the address, which
means the server binds to all network interfaces, on the specified port. This
might be fine or it might not be in your specific case. If you need the server
to bind to a specific IP address, specify that instead of `0.0.0.0`.
</div>

#### The storage settings (optional)

The default storage settings URL is `memory://`, which causes the server to
store session state in-memory, which is ephemeral - i.e., on server crash or
shutdown, any existing session state is lost. If you don't have a `[storage]`
section in your configuration, then `memory://` is used.

##### Using in-memory session state

```shell
cd nl-wallet
export TARGET_DIR=target/is-config && mkdir -p "$TARGET_DIR/parts"
cat <<EOF > "$TARGET_DIR/parts/10-storage-settings.toml"

[storage]
url = "memory://"
EOF
```

##### Using database persisted session state

```shell
cd nl-wallet
export TARGET_DIR=target/is-config && mkdir -p "$TARGET_DIR/parts"
cat <<EOF > "$TARGET_DIR/parts/10-storage-settings.toml"

[storage]
url = "postgres://$DB_USERNAME:$DB_PASSWORD@$PGHOST:$PGPORT/$DB_NAME"
EOF
```

<div class="admonition warning">
<p class="title">Make sure database setting environment variables are set</p>
When you use the `postgres://` URL, you tell the server to store session state
in a PostgreSQL database. In the above, we assume you still have the environment
variables configured like we documented in the database configuration section
(i.e., [Using a database backend](#setting-up-postgresql)).
</div>

#### Configuring a hardware security module (optional)

You can opt to use a hardware security module (HSM) to store private keys for
`disclosure_settings` and `attestation_settings`. To do so we need to configure
a few things:

```shell
cd nl-wallet
export TARGET_DIR=target/is-config && mkdir -p "$TARGET_DIR/parts"
cat <<EOF > "$TARGET_DIR/parts/11-hardware-security-module.toml"

[hsm]
library_path = "/path/to/some/pkcs11-compatible/library.so"
user_pin = "12345678"
max_sessions = 3
max_session_lifetime_in_sec = 900
EOF
```

<div class="admonition note">
<p class="title">Make sure you specify a correct library path</p>
The HSM functionality depends on a PKCS#11 compatible shared library which will
have been provided by your HSM vendor. Technically you can also use any PKCS#11
implementation here. For development purposes we test with the [softhsm2][28]
library, which is usually called something like `libsofthsm2.so` (the path
location and filename extension differs per operating system and/or packaging
environment).
</div>

<div class="admonition note">
<p class="title">Private key field needs to be a key label when using HSM type</p>
<p>When using a hardware security module, the `private_key` field of
`disclosure_settings` and/or `attestation_settings` need to be the HSM key
label.</p>
<p>It is possible to use *both* hardware *and* software private keys in the same
`issuance_server` instance. Simply make sure you set `private_key_type` to
`hsm` for HSM managed keys and to `software` when using base64 encoded DER
strings in the `private_key` field.</p>
</div>

#### Configuring the status list settings (optional)


The issuance_server maintains binary format token status lists which are used
for revocation and validity checking. We'll set the defaults here:

```shell
cd nl-wallet
export TARGET_DIR=target/is-config && mkdir -p "$TARGET_DIR/parts"
cat <<EOF > "$TARGET_DIR/parts/12-status-lists.toml"

[status_lists]
list_size = 100_000
create_threshold = 1000
EOF
```

<div class="admonition note">
<p class="title">Using a `storage_url` under `[status_lists]`</p>
<p>By default the status lists uses the same database as configured for the
session store. If you have a memory store as session store you need to configure
a PostgreSQL `storage_url` under the `[status_lists]` block. You can also
configure a `storage_url` under the `[status_lists]` block if you want to store
the status list in a different database or use a different user.</p>
</div>

#### Configuring disclosure-based issuance elements

We're now going to configure the configuration blocks that together make up a
disclosure-based issuance element. We're going to call our `insurance` elements
as follows:

  * `[disclosure_settings.insurance]`
  * `[[disclosure_settings.insurance.dcql_query.credentials]]`
  * `[disclosure_settings.insurance.attestation_url_config]`
  * `[attestation_settings.insurance]`

In the next sub-sections we'll cover each one of these.

##### The disclosure settings

We're going to base64 encode the reader key and certificate within the
`private_key` and `certificate` fields of the `disclosure_settings`. This is
the certificate that embedded the previously created `reader_auth.json`. Let's
create the section:

```shell
cd nl-wallet
export BASE64="openssl base64 -e -A"
export IDENTIFIER="foocorp"
export TARGET_DIR=target/is-config && mkdir -p "$TARGET_DIR/parts"
cat <<EOF > "$TARGET_DIR/parts/13-disclosure-settings.toml"

[disclosure_settings.insurance]
private_key_type = "software"
private_key = "$(< "${TARGET_DIR}/reader.${IDENTIFIER}.key.der" $BASE64)"
certificate = "$(< "${TARGET_DIR}/reader.${IDENTIFIER}.crt.der" $BASE64)"
EOF
unset BASE64 IDENTIFIER
```

##### The DCQL query credentials

Add the [DCQL query credentials][29] (note, the double brackets, i.e., `[[`,
and `]]`, are intentional):

```shell
cd nl-wallet
export TARGET_DIR=target/is-config && mkdir -p "$TARGET_DIR/parts"
cat <<EOF > "$TARGET_DIR/parts/14-dcql-query-credentials.toml"

[[disclosure_settings.insurance.dcql_query.credentials]]
id = "insurance_credential"
format = "dc+sd-jwt"
meta = { vct_values = ["urn:eudi:pid:nl:1"] }
claims = [ { path = ["urn:eudi:pid:nl:1", "bsn"], intent_to_retain = false } ]
EOF
```

##### The attestation URL configuration

The attestation URL configuration section configures where the `issuance_server`
is expected to fetch its attestable attributes from. The `base_url` setting
points to the attestation server (when you run in a local development
environment, this is the `demo_issuer`, running on port `3006`):

```shell
cd nl-wallet
export TRUST_ANCHORS=()
for i in scripts/devenv/target/demo_issuer/ca.crt.der target/ca-cert/ca.*.crt.der; do \
    [[ -f $i ]] && TRUST_ANCHORS+=($(openssl base64 -e -A -in $i)); done

export TARGET_DIR=target/is-config && mkdir -p "$TARGET_DIR/parts"
cat <<EOF > "$TARGET_DIR/parts/15-attestation-url-config.toml"

[disclosure_settings.insurance.attestation_url_config]
base_url = "https://your-attestation-server.example.com/insurance"
trust_anchors = [$(printf '"%s",' "${TRUST_ANCHORS[@]}" | sed 's/,$//')]
EOF
unset TRUST_ANCHORS
```

##### The attestation settings

We're now going to base64 encode the issuer key and certificate within the
`private_key` and `certificate` fields of the `attestation_settings`. This is
the certificate that embedded the previously created `issuer_auth.json`. Let's
create the section:

```shell
cd nl-wallet
export BASE64="openssl base64 -e -A"
export IDENTIFIER="foocorp"
export TARGET_DIR=target/is-config && mkdir -p "$TARGET_DIR/parts"
cat <<EOF > "$TARGET_DIR/parts/16-attestation-settings.toml"

[attestation_settings.insurance]
valid_days = 365
copies_per_format = { "mso_mdoc" = 4, "dc+sd-jwt" = 4 }
private_key_type = "software"
private_key = "$(< "${TARGET_DIR}/issuer.${IDENTIFIER}.key.der" $BASE64)"
certificate = "$(< "${TARGET_DIR}/issuer.${IDENTIFIER}.crt.der" $BASE64)"
EOF
```

##### The attestation settings associated status list settings

Next to the previously done `[status_lists]` settings, which control how big and
when status lists are created, an `attestation_settings` associated
`status_list` block configures the `base_url`, which is added to the attestation
as a location for the Status List Claim.

This needs to be publicly reachable by the NL Wallet app and the verifiers.
Additionally, the `private_key` settings are used to sign the Status Token List,
which the wallet and the `verification_server` validate when interacting with
Status Token Lists. The service pointed to by `base_url` needs to be publicly
reachable by the NL Wallet app and the verifiers. Let's create it:

```shell
cd nl-wallet
export BASE64="openssl base64 -e -A"
export IDENTIFIER="foocorp"
export TARGET_DIR=target/is-config && mkdir -p "$TARGET_DIR/parts"
cat <<EOF > "$TARGET_DIR/parts/17-attestation-status-list.toml"

[attestation_settings.insurance.status_list]
base_url = "https://cdn.example.com/tsl"
private_key_type = "software"
private_key = "$(< "${TARGET_DIR}/tsl.${IDENTIFIER}.key.der" $BASE64)"
certificate = "$(< "${TARGET_DIR}/tsl.${IDENTIFIER}.crt.der" $BASE64)"
EOF
```

#### Writing the configuration file

In the previous sections, you've created a bunch of partial configuration blocks
which we will use in this section to generate our `issuance_server.toml`
configuration file. To generate our configuration file, issue the following
command:

```shell
cd nl-wallet
export TARGET_DIR=target/is-config && mkdir -p "$TARGET_DIR/parts"
cat "$TARGET_DIR"/parts/*.toml > "$TARGET_DIR/issuance_server.toml"
```

You should now have a configuration file in the `$TARGET` directory called
`issuance_server.toml`. Feel free to check the file to see if everything
looks like you'd expect.

### Running the server for the first time

In section [Obtaining the software](#obtaining-the-software) we have described
how you can obtain the software. In this section, we assume you have a Linux
AMD64 static executable called `issuance_server` that you can run. We're going
to `cd` into the `target/is-config` directory, and we assume the binary exists
there (it does if you [followed along](#obtaining-the-software) previously):

```shell
cd nl-wallet/target/is-config
./issuance_server
```

If all went well, the server is now running and ready to serve requests. To test
the service, you can send session initiation and status requests to it (check
out the [API specifications](#issuer-api-specifications) section for how to do
that).

Make sure to consider your [logging settings](#logging-settings-optional) if you
need to troubleshoot.

### Validating your setup

During startup, the `issuance_server` performs some checks on the configuration
to prevent common configuration problems. Most notably the following checks are
performed:

- Verify all `disclosure_settings` and `attestation_settings` certificates are
  valid;
- Verify all `disclosure_settings` and `attestation_settings` certificates are
  signed by any of the `reader_trust_anchors` and `issuer_trust_anchors`;
- Verify all `disclosure_settings` certificates are valid reader-certificates,
  and contain the necessary Extended Key Usages and the `reader_auth.json`;
- Verify all `attestation_settings` certificates are valid issuer-certificates,
  and contain the necessary Extended Key Usages and the `issuer_auth.json`;
- Verify all `disclosure_settings` and `attestation_settings` key-pairs are
  valid, i.e., the public and private keys should belong together;

If this process discovers any configuration errors, the application will report
an error and abort. For more insights into this process,
[enable logging](#logging-settings-optional).

## Issuer API specifications

The API specifications for the issuer endpoints are available in the
`wallet_docs/openapi` part of of the git repository.

Have a look at the [OpenAPI Specifications][30] section to learn how to open
and use these.

## Integrating your app with your issuance server

The wallet starts disclosure based issuance if it encounters a UL (or a QR with
a UL within it) of a specific format. To create this UL, proceed as follows.

  1. If your `issuance_server` is reachable on the internet by the wallet at
     `https://issuer.example.com`, create a URL of the following form:

     ```
     https://issuer.example.com/disclosure/foo/request_uri?session_type=same_device
     ```

     In the above URL, `foo` has to be the identifier you used when you
     configured the disclosure-based issuance settings in the configuration
     file (so, it relates to `[disclosure_settings.foo]`,
     `[disclosure_settings.foo.dcql_query.credentials]`, and
     `[disclosure_settings.foo.attestation_url_config]`)

  2. URL-encode the above URL.

  3. Create the UL as follows (newlines only for readability purposes):

     ```
     https://app.example.com/deeplink/disclosure_based_issuance
       ?request_uri_method=post
       &client_id=disclosure_based_issuance.example.com
       &request_uri=https%3A%2F%2Fissuer.example.com...
     ```

     In which the `client_id` has to be the SAN DNS name from the RP
     `certificate`, and the `request_uri` is the URL-encoded URL from
     the previous step.

  4. Place this UL on your website (within in a QR code in case of cross device
     flows).

## References

Below you'll find a collection of links which we reference to through the
entire text. Note that they don't display when rendered within a website, you
need to read the text in a regular text editor or pager to see them.

[1]: https://github.com/minbzk/nl-wallet
[2]: https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html
[3]: https://www.logius.nl/onze-dienstverlening/toegang/digid
[4]: https://github.com/minvws/nl-rdo-max
[5]: https://www.rvig.nl/basisregistratie-personen
[6]: https://edi.pleio.nl/news/view/93f40956-3671-49c9-9c82-2dab636b59bf/psasad-documenten-nl-wallet
[7]: ../architecture/use-cases/issuance-with-openid4vci
[8]: ../architecture/use-cases/disclosure-based-issuance
[9]: create-a-verifier
[11]: ../community/onboarding
[17]: https://docs.rs/env_logger/latest/env_logger/#enabling-logging
[19]: https://github.com/MinBZK/nl-wallet#user-content-development-requirements
[20]: https://github.com/MinBZK/nl-wallet/releases
[22]: https://www.postgresql.org/download/
[23]: https://www.docker.com/
[24]: https://github.com/MinBZK/nl-wallet/blob/main/wallet_core/wallet_server/issuance_server/issuance_server.example.toml
[25]: https://developer.apple.com/documentation/xcode/supporting-associated-domains
[26]: https://developer.android.com/training/app-links
[27]: ../community/create-a-ca
[28]: https://github.com/softhsm/SoftHSMv2
[29]: https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#name-digital-credentials-query-l
[30]: ../development/openapi-specifications
[31]: create-a-verifier
