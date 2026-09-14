# Authorized attributes

On this page you'll find a collection of so-called claim paths, which are
defined within a number of verifiable credential types (or doctypes in mdoc) we
support. These claim paths are used to indicate what values you're interested
in. They are used within the `credentials` field of a Wallet Relying Party
Registration Certificate.

## How authorized attributes are specified

The `credentials` field contains an array of credential authorizations. Each
entry authorizes one credential format and identifies the credential type in
its `meta` field. Its optional `claim` field lists the claim paths that the
relying party may request. Note that the registration certificate uses the
singular name `claim`, while a DCQL query uses `claims`.

For example, the following entries authorize the `given_name` and
`family_name` claims from the Dutch PID in both mdoc and SD-JWT form:

```json
{
  "credentials": [
    {
      "format": "mso_mdoc",
      "meta": { "doctype_value": "urn:eudi:pid:nl:1" },
      "claim": [
        { "path": ["urn:eudi:pid:nl:1", "given_name"] },
        { "path": ["urn:eudi:pid:nl:1", "family_name"] }
      ]
    },
    {
      "format": "dc+sd-jwt",
      "meta": { "vct_values": ["urn:eudi:pid:nl:1"] },
      "claim": [
        { "path": ["given_name"] },
        { "path": ["family_name"] }
      ]
    }
  ]
}
```

For every credential query in an OpenID4VP request, the wallet requires one
entry in the registration certificate that authorizes the complete credential
query:

- The `format` values must match.
- An mdoc `doctype_value` must match exactly. Every SD-JWT `vct_values` value
  in the query must occur in the registration certificate entry.
- Every requested claim path must occur in that same entry. Authorizations from
  multiple entries cannot be combined to cover one credential query.
- A claim authorization can use `values` to restrict the values that may be
  requested. In that case, the query must request a non-empty subset of those
  values. Without `values`, the claim authorization does not restrict values.

The wallet rejects the complete disclosure request when any credential query
falls outside these authorizations.

### Support for both SD-JWT- and mdoc-style authorized attributes

An mdoc and an SD-JWT representation of the same credential require separate
entries in the registration certificate. Their claim paths use different
conventions and are matched exactly; the wallet does not translate between
them during authorization.

An mdoc claim path starts with its namespace followed by the element
identifier, for example `["urn:eudi:pid:nl:1", "given_name"]`. Elements in a
separate namespace use that namespace instead, for example
`["urn:eudi:pid:nl:1.address", "street_address"]`.

An SD-JWT claim path follows the nesting of the JSON claims. A top-level path
is written as `["given_name"]`, while a nested path is written as
`["address", "street_address"]`.

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
<p>The below tables are generated using our `authorized-attributes-tables.sh`
script, which parses our `scripts/devenv/eudi:*.json` documents. If you suspect
the claim path attributes you're looking at might be out-of-date, you can invoke
`authorized-attributes-tables.sh` and make sure you're looking at the latest we
support.</p>
<p>Do note that, while we use the aforementioned JSON definition documents in
our locally running `pid_issuer`, it is not guaranteed that a `pid_issuer`
running in one of our live environments uses these exact-same JSON definition
documents. When you work with one of our live environments, and thus followed
our [onboarding][1] procedure, you can contact our operations team to obtain
the necessary information.</p>
</div>

### Claims in eudi:pid:1

| Claim Path             | Label         | Description                                       | Language |
|------------------------|---------------|---------------------------------------------------|----------|
| age_over_18            | Over 18       | Whether the person is over 18                     | en-US    |
| birthdate              | Birth date    | Birth date of the person                          | en-US    |
| family_name            | Name          | Family name of the person, including any prefixes | en-US    |
| given_name             | First name    | First name of the person                          | en-US    |
| nationalities          | Nationalities | List of nationalities of the person               | en-US    |
| address.country        | Country       | Country of the address                            | en-US    |
| address.house_number   | House number  | House number of the address                       | en-US    |
| address.locality       | City          | City of the address                               | en-US    |
| address.postal_code    | Postal code   | Postal code of the address                        | en-US    |
| address.street_address | Street        | Street of the address                             | en-US    |

### Claims in eudi:pid:nl:1

| Claim Path             | Label           | Description                                       | Language |
|------------------------|-----------------|---------------------------------------------------|----------|
| age_over_18            | Over 18         | Whether the person is over 18                     | en-US    |
| age_over_18            | 18+             | Of de persoon 18+ is                              | nl-NL    |
| birthdate              | Birth date      | Birth date of the person                          | en-US    |
| birthdate              | Geboortedatum   | Geboortedatum van de persoon                      | nl-NL    |
| bsn                    | BSN             | BSN of the person                                 | en-US    |
| bsn                    | BSN             | BSN van de persoon                                | nl-NL    |
| family_name            | Name            | Family name of the person, including any prefixes | en-US    |
| family_name            | Achternaam      | Achternaam van de persoon, inclusief voorvoegsels | nl-NL    |
| given_name             | First name      | First name of the person                          | en-US    |
| given_name             | Voornaam        | Voornaam van de persoon                           | nl-NL    |
| nationalities          | Nationalities   | List of nationalities of the person               | en-US    |
| nationalities          | Nationaliteiten | Lijst van nationaliteiten van de persoon          | nl-NL    |
| recovery_code          | Recovery code   | Recovery code of the person                       | en-US    |
| recovery_code          | Herstelcode     | Herstelcode van de persoon                        | nl-NL    |
| address.country        | Country         | Country of the address                            | en-US    |
| address.country        | Land            | Land van het adres                                | nl-NL    |
| address.house_number   | House number    | House number of the address                       | en-US    |
| address.house_number   | Huisnummer      | Huisnummer van het adres                          | nl-NL    |
| address.locality       | City            | City of the address                               | en-US    |
| address.locality       | Stad            | Stad van het adres                                | nl-NL    |
| address.postal_code    | Postal code     | Postal code of the address                        | en-US    |
| address.postal_code    | Postcode        | Postcode van het adres                            | nl-NL    |
| address.street_address | Street          | Street of the address                             | en-US    |
| address.street_address | Straatnaam      | Straatnaam van het adres                          | nl-NL    |

<!-- References -->

[1]: ../community/onboarding
