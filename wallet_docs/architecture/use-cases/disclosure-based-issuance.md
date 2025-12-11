# Disclosure-based Issuance

Disclosure-based issuance allows an attestation issuer to issue attestations to
a wallet based on the contents of attributes disclosed by that same wallet.

The flow starts when the user scans a QR code, or taps on a UL, that starts the
disclosure-based issuance flow.

The wallet presents a disclosure screen to the user, which upon user consent is
followed immediately by an issuance screen presenting the new attestations. In
between the two screens, the issuer receives the disclosed attributes, and can
use those to determine the contents of the attestation(s) to be issued.

The issuer side of this functionality is implemented by the `issuance_server`
binary, which is a part of the `wallet_server` crate.

Technically, disclosure-based issuance works like this:

  1. At the end of the disclosure session, the `issuance_server` sends the
     disclosed attributes to a preconfigured HTTP endpoint, at which the issuer
     must run an HTTP server that must respond with the attestations to be
     issued. This server is called the *attestation server*.

  2. The `issuance_server` starts an OpenID4VCI session in the [Pre-Authorized
     Code Flow][1], puts the Pre-Authorized Code into an [OpenID4VCI Credential
     Offer][2], and URL-encodes that into the `redirect_uri` that gets sent to
     the wallet.

  3. Using the Credential Offer with the Pre-Authorized Code within it, the
     wallet performs the OpenID4VCI session with the `issuance_server`.

## Attestation server

The *attestation server* exchanges disclosed attributes for attestations to be
issued. It is contacted by the `issuance_server` during a disclosure-based
issuance session to do this. Note that this server does not need to be reachable
by the wallet, and it should probably not be connected to the internet directly.

The *attestation server* receives an HTTP `POST` by the `issuance_server`,
containing a JSON structure of the same type as the `/disclosed_attributes`
endpoint of the `verification_server` (you can have a look at our
[openapi specifications](../../development/openapi-specifications) too).

Here's an example of the JSON structure, which in Rust is an
`Vec<DisclosedAttestations>`. For example:

```http
POST / HTTP/1.1
Content-Type: application/json
Accept: */*
Host: localhost:51560
Content-Length: 267

[
  {
    "id": "my_credential",
    "attestations": [
      {
        "attestation_type": "com.example.pid",
        "attributes": {
          "com.example.pid": {
            "bsn": "999991772"
          }
        },
        "issuer_uri": "https://cert.issuer.example.com/",
        "attestation_qualification": "QEAA",
        "ca": "ca.pid.example.com",
        "issuance_validity": {
          "signed": "2025-04-15T07:02:26Z",
          "valid_from": "2025-04-15T07:02:26Z",
          "valid_until": "2026-04-15T07:02:26Z"
        }
      }
    ]
  }
]
```

The *attestation server* must respond with a JSON-serialized
`Vec<IssuableDocument>`. For example:

```http
HTTP/1.1 200 OK
Content-Type: application/json

[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "attestation_type": "com.example.degree",
    "attributes": {
      "university": "Example university",
      "education": "Example education",
      "graduation_date": "1970-01-01",
      "grade": "A",
      "cum_laude": true
    }
  }
]
```

The JSON array may be empty (i.e. `[]`), to indicate that no attestations to be
issued were found. In that case, the wallet will present a screen to the user
explaining that there were no attestations found that can be issued.

The `id` of the attestation is used to internally link the status claims of one
or more status list to the attestation that will be issued. This `id` will not
be included in the attestation. It can be used to revoke the attestation later.

## Sequence diagram

This diagram shows how we compose OpenID4VP with OpenID4VCI in a pre-authorized
code flow to issue attestations based on the contents of other attestations.

The flow starts with the user scanning a QR code or tapping a UL that contains
the URL to the disclosure-based issuer. When the wallet contacts the issuer
using this URL, it starts a disclosure session requesting some preconfigured
set of attributes that will allow it to determine the attributes that it wants
to issue.

For example, suppose the issuer wishes to issue academic degrees based on the
user's BSN; then it would preconfigure an Authorization Request that requests
the BSN from the PID.

After the user has disclosed the requested attributes, the issuer sends those to
an internal endpoint that responds with the attestations to be issued. In the
earlier mentioned example, this would be a database containing academic degrees
indexed by BSNs.

Finally, the issuer issues the attestations using OpenID4VCI in a pre-authorized
code flow.

During OpenID4VCI, the issuer requires the wallet to include the keys of the
attestations that it disclosed earlier in its Proof of Association (PoA) when
it sends its Proofs of Possession (PoPs) for the keys of the attestation
(copies).

This enforces that the newly issued attestations are bound to the same Wallet
Secure Cryptographic Device (WSCD, see [section 4.3.2, Components of a Wallet
Unit][3]) as the one that disclosed the attestations in the first part of the
protocol.

```{mermaid}
sequenceDiagram
    autonumber
    actor User
    participant App as Wallet
    participant Issuer as Disclosure-based issuer
    participant IssuerDataProvider as Data provider

    User->>+App: Invoke UL (https://wallet_ul/disclosure_based_issuance)
    App->>+Issuer: POST to request URI [OpenID4VP]
    Issuer->>Issuer: Start disclosure session with preconfigured request
    Issuer->>-App: Authorization Request [OpenID4VP]

    App ->>-User: Request approval for disclosure
    User->>+ App: Approve disclosure with PIN

    App->>+Issuer: Authorization Response [OpenID4VP] (including PoA) *
    Issuer->>Issuer: Verify disclosed attributes
    Issuer->>+IssuerDataProvider: Disclosed attributes
    IssuerDataProvider->>-Issuer: Respond with attestations to issue
    Issuer->>Issuer: Start issuance session
    Issuer->>Issuer: Create OpenID4VCI Credential Offer (pre-authorized code flow)
    Issuer->>-App: Redirect URI [OpenID4VP]

    App->>App: Parse Token Request from Credential Offer
    App->>+Issuer: Pre-authorized code [OpenID4VCI]
    Issuer->>-App: Access Token + attestation previews [OpenID4VCI]

    App->>-User: Request approval for attestations
    User->>+App: Approve attestation issuance with PIN

    App->>+Issuer: Access token [OpenID4VCI]
    Issuer->>Issuer: Enforce disclosed attestation keys in PoA (from Step 7 *)
    Issuer->>-App: Attestations [OpenID4VCI]
    deactivate App
```

## References

Below you'll find a collection of links which we reference to through the
entire text. Note that they don't display when rendered within a website, you
need to read the text in a regular text editor or pager to see them.

[1]: https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-final.html#name-pre-authorized-code-flow
[2]: https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-final.html#name-credential-offer
[3]: https://eudi.dev/latest/architecture-and-reference-framework-main/#43-reference-architecture
