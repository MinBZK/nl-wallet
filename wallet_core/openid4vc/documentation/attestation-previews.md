# Attestation previews during issuance

In the NL Wallet, during the issuance process of PID/(Q)EAA's to the wallet we wish to show previews of the attestations including attribute values in the wallet just before issuance is completed. The purpose of this is inform the user of the attribute values they are going to receive, and to grant them the ability to check the values and refuse if they are not correct. Since these attestation previews are just meant to inform the user they should be unsigned, i.e., not be fully functional (Q)EAA or PID instances.

The NL Wallet uses the [OpenID4VCI issuance protocol](https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html), which does not offer such a preview. It does support previews without attribute values early on in the protocol, and the OAuth server dealing with the Authorization Request and Response may show the full attestation previews including attribute values after it has identified the user, during the phase of the protocol where it has control over the flow in the browser. However, instead of this we wish to show attestation previews including attribute values in the NL Wallet, so that we can offer a UX for viewing the attestation previews that is consistent over all (future) issuers and does not depend on if and how they show such a preview in the browser.

Therefore, the NL Wallet implements a custom extension to the OpenID4VCI protocol that adds attestation previews to it. This extension is documented here.

## Protocol flow

In the OpenID4VCI protocol, the wallet and issuer exchange the following messages:
- Authorization Request and Authorization Response, in case of the Authorized Code Flow;
- Token Request and Token Response, endowing the wallet with the access token that grants access to the Credential endpoint;
- Credential Request and Credential Response, in which the wallet exchanges the access token for the attestations.

In this extension, the attestation previews are sent by the issuer to the wallet as an extra field added to the Token Response JSON object.

## Data structure

The extra field in the Token Response is an array called `credential_previews`, which contains objects representing previews of the attestations being issued. Each preview object contains a `format` field containing a string identifying the format of the attestation. The remaining fields of the object may differ per format.

In the case of the `mso_mdoc` format value, which is the only currently supported format implemented by the NL Wallet, the remaining fields in the attestation preview object are as follows.

- `issuer`: String containing the Base64-encoded DER-encoded X.509 certificate of the issuer with which the attestation will be signed, i.e., a certificate complying with the profile from Table B.3 in Appending B.1.4 of ISO 18013-5. This contains among others the name of the issuer.
- `unsigned_mdoc`: Object representing the attestation itself, containing:
  - `doctype`: String containing the doctype of the attestation, as defined in ISO 18013-5.
  - `valid_from`: String containing in ISO 8601 format the date and time from which this attestation is valid.
  - `valid_until`: String containing in ISO 8601 format the date and time until which this attestation is valid.
  - `copy_count`: Integer specifying how many copies of this attestation the wallet will receive (for unlinkability purposes).
  - `attributes`: Object in which the keys are the namespaces, and the values are arrays representing the attributes, which are objects containing the following:
    - `name`: String containing the name of the attribute.
    - `value`: JSON value containing the attribute value.

## Example

An example of a Token Response containing an attestation preview is shown below.

```json
{
  "access_token": "lKb0I2Fr1ID747bDatmzHdqUKsC1BS8MY",
  "bearer_type": "DPoP",
  "c_nonce": "xDp1LxNtx8KCqXdxSzQpAHqElFZVTXMX",
  "credential_previews": [
    {
      "format": "mso_mdoc",
      "issuer": "MIJumzCCbkKgAwIBAgIVAPrmBbCgP...",
      "unsigned_mdoc": {
        "doctype": "com.example.pid",
        "valid_from": "2024-04-12T12:47:06Z",
        "valid_until": "2025-04-12T12:47:06Z",
        "copy_count": 2,
        "attributes": {
          "com.example.pid": [
            {
              "name": "bsn",
              "value": "999991772"
            },
            {
              "name": "family_name",
              "value": "De Bruijn"
            },
            {
              "name": "given_name",
              "value": "Willeke Liselotte"
            },
            {
              "name": "birth_date",
              "value": "1997-05-10"
            },
            {
              "name": "age_over_18",
              "value": true
            },
            {
              "name": "birth_country",
              "value": "NL"
            },
            {
              "name": "birth_state",
              "value": "Zuid-Holland"
            },
            {
              "name": "birth_city",
              "value": "Delft"
            },
            {
              "name": "gender",
              "value": 2
            },
            {
              "name": "nationality",
              "value": "NL"
            }
          ]
        }
      }
    }
  ]
}
```
