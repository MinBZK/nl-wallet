# Authorized attributes

In this document you'll find a collection of so-called claim paths, which are
defined within a number of verifiable credential types (or doctypes in MDOC) we
support. These claim paths are used to indicate what values you're interested
in. They are used within the `authorizedAttributes` object in `reader_auth.json`
documents.

## How authorized attributes are specified

As mentioned, in the `reader_auth.json` document, you have an object which
indicates what attributes you want to verify, called `authorizedAttributes`.

In the `scripts/devenv` subdirectory of our git repository, you'll find various
`*_reader_auth.json` example documents which all include an
`authorizedAttributes`.

Let's take the (quite extensive) one from our "XYZ Bank" as an example:

```json
"authorizedAttributes": {
    "urn:eudi:pid:nl:1": [
      ["urn:eudi:pid:nl:1", "given_name"],
      ["urn:eudi:pid:nl:1", "family_name"],
      ["urn:eudi:pid:nl:1", "birthdate"],
      ["urn:eudi:pid:nl:1", "bsn"],
      ["given_name"],
      ["family_name"],
      ["birthdate"],
      ["bsn"]
    ],
    "urn:eudi:pid:1": [
      ["given_name"],
      ["family_name"],
      ["birthdate"]
    ],
    "urn:eudi:pid-address:nl:1": [
      ["urn:eudi:pid-address:nl:1.address", "street_address"],
      ["urn:eudi:pid-address:nl:1.address", "house_number"],
      ["urn:eudi:pid-address:nl:1.address", "postal_code"],
      ["address", "street_address"],
      ["address", "house_number"],
      ["address", "postal_code"]
    ],
    "urn:eudi:pid-address:1": [
      ["address", "street_address"],
      ["address", "house_number"],
      ["address", "postal_code"]
    ]
  }
```

The object contains named array values; the name represents a doctype (in MDOC
parlance) or VCT (Verifiable Credential Type) in SD-JWT speak, with a verion
number (i.e., the `1`'s after the name) appended. In the above example:
`urn:eudi:pid:nl:1`, `urn:eudi:pid:1"`, `urn:eudi:pid-address:nl:1`, and
`urn:eudi:pid-address:1` are doctype/vct names with an appended version `1`.

These names refer to a document which defines what values a given doctype/vct
can contain; the tables below which show you what each doctype/vct can contain
are generated from those documents.

### Support for both SD-JWT and MDOC style authorized attributes

The array contains one or more arrays with so-called "claim paths". In the
example above, you'll find that some values seem a bit redundant, like:

```json
[
  ["urn:eudi:pid-address:nl:1.address", "street_address"],
  ["address", "street_address"]
]
```

In this case, the first (somewhat wordy) specification is an MDOC style path
specification, and the second shorter one is an SD-JWT claim path (which in the
tables below is shown as `address.street_address`). Specifying this in both
forms allows the verifier (or issuer doing disclosure-based-issuance) to accept
both MDOC and SD-JWT style attributes.

### A note about extended VCTs

In the VCT definition documents (in our git repository, the JSON documents that
define our VCT/doctypes are `scripts/devenv/eudi:*.json`) there is an optional
`extends` keyword that can indicate a parent VCT, allowing a VCT to be
"extended".

For example, `urn:eudi:pid:nl:1` extends `urn:eudi:pid:1` with `bsn` and
`recovery_code`, in addition to Dutch language labels and descriptions for all
claim paths. In `urn:eudi:pid:nl:1` you'll see an `extends` element which points
to `urn:eudi:pid:1`.

## Overview of supported authorized attributes

Below you'll find a few tables which show which attributes we support, in some
cases in both English and Dutch form.

<div class="admonition note"><p class="title">These tables are generated</p>
<p>The below tables are generated using our `authorized-attributes.sh` script,
which parses our `scripts/devenv/eudi:*.json` documents. If you suspect the
claim path attributes you're looking at might be out-of-date, you can invoke
`supported-attributes.sh` and make sure you're looking at the latest we support.
</p></div>

### Claims in eudi:pid:1

| Claim Path    | Label         | Description                                       | Language |
| ------------- | ------------- | ------------------------------------------------- | -------- |
| age_over_18   | Over 18       | Whether the person is over 18                     | en-US    |
| birthdate     | Birth date    | Birth date of the person                          | en-US    |
| family_name   | Name          | Family name of the person, including any prefixes | en-US    |
| given_name    | First name    | First name of the person                          | en-US    |
| nationalities | Nationalities | List of nationalities of the person               | en-US    |

### Claims in eudi:pid:nl:1

| Claim Path    | Label           | Description                                       | Language |
| ------------- | --------------- | ------------------------------------------------- | -------- |
| age_over_18   | Over 18         | Whether the person is over 18                     | en-US    |
| age_over_18   | 18+             | Of de persoon 18+ is                              | nl-NL    |
| birthdate     | Birth date      | Birth date of the person                          | en-US    |
| birthdate     | Geboortedatum   | Geboortedatum van de persoon                      | nl-NL    |
| bsn           | BSN             | BSN of the person                                 | en-US    |
| bsn           | BSN             | BSN van de persoon                                | nl-NL    |
| family_name   | Name            | Family name of the person, including any prefixes | en-US    |
| family_name   | Achternaam      | Achternaam van de persoon, inclusief voorvoegsels | nl-NL    |
| given_name    | First name      | First name of the person                          | en-US    |
| given_name    | Voornaam        | Voornaam van de persoon                           | nl-NL    |
| nationalities | Nationalities   | List of nationalities of the person               | en-US    |
| nationalities | Nationaliteiten | Lijst van nationaliteiten van de persoon          | nl-NL    |
| recovery_code | Recovery code   | Recovery code of the person                       | en-US    |
| recovery_code | Herstelcode     | Herstelcode van de persoon                        | nl-NL    |

### Claims in eudi:pid-address:1

| Claim Path             | Label        | Description                 | Language |
| ---------------------- | ------------ | --------------------------- | -------- |
| address.country        | Country      | Country of the address      | en-US    |
| address.house_number   | House number | House number of the address | en-US    |
| address.locality       | City         | City of the address         | en-US    |
| address.postal_code    | Postal code  | Postal code of the address  | en-US    |
| address.street_address | Street       | Street of the address       | en-US    |

### Claims in eudi:pid-address:nl:1

| Claim Path             | Label        | Description                 | Language |
| ---------------------- | ------------ | --------------------------- | -------- |
| address.country        | Country      | Country of the address      | en-US    |
| address.country        | Land         | Land van het adres          | nl-NL    |
| address.house_number   | House number | House number of the address | en-US    |
| address.house_number   | Huisnummer   | Huisnummer van het adres    | nl-NL    |
| address.locality       | City         | City of the address         | en-US    |
| address.locality       | Stad         | Stad van het adres          | nl-NL    |
| address.postal_code    | Postal code  | Postal code of the address  | en-US    |
| address.postal_code    | Postcode     | Postcode van het adres      | nl-NL    |
| address.street_address | Street       | Street of the address       | en-US    |
| address.street_address | Straatnaam   | Straatnaam van het adres    | nl-NL    |
