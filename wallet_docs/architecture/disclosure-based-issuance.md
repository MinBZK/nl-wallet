# Disclosure-based Issuance

Disclosure based issuance allows an attestation issuer to issue attestations to a wallet based on the contents of attributes disclosed by that same wallet.

The flow starts when the user scans a QR code, or taps on a UL, that starts the disclosure based issuance flow.
The wallet then presents a disclosure screen to the user, which upon user consent is followed immediately by an issuance screen presenting the new attestations.
In between the two screens, the issuer receives the disclosed attributes, and can use those to determine the contents of the attestation(s) to be issued.

The issuer side of this functionality is implemented by the `issuance_server` binary under `wallet_core/wallet_server`.

Technically, disclosure based issuance is achieved as follows:

1. At the end of the disclosure session, the `issuance_server` sends the disclosed attributes to a preconfigured HTTP endpoint, at which the issuer must run an HTTP server that must respond with the attestations to be issued. This server is called the *attestation server*.
2. The `issuance_server` starts an OpenID4VCI session in the [Pre-Authorized Code Flow](https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-13.html#name-pre-authorized-code-flow), puts the Pre-Authorized Code into an [OpenID4VCI Credential Offer](https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-13.html#name-credential-offer), and URL-encodes that into the `redirect_uri` that gets sent to the wallet.
3. Using the Credential Offer with the Pre-Authorized Code within it, the wallet performs the OpenID4VCI session with the `issuance_server`.

## API

### Attestation server

The attestation server exchanges disclosed attributes for attestations to be issued. It is contacted by the `issuance_server` during a disclosure based issuance session to do this. Note that this server does not need to be reachable by the wallet, and it probably should not be directly connected to the internet.

The attestation server receives an HTTP `POST` by the `issuance_server` containing a JSON structure of the same form as the `/disclosed_attributes` endpoint of the `verification_server` returns, as documented in the [verifier documentation](../get-started/create-a-verifier.md#retrieve-disclosure-results). More specifically, this is a JSON-serialized `IndexMap<String, DocumentDisclosedAttributes>`. For example:

```http
POST / HTTP/1.1
Content-Type: application/json
Accept: */*
Host: localhost:51560
Content-Length: 267

{
  "my_credential": [
    {
      "attestationType": "com.example.pid",
      "attributes": {
        "com.example.pid": {
          "bsn": "999991772"
        }
      },
      "issuerUri": "https://cert.issuer.example.com/",
      "ca": "ca.pid.example.com",
      "validityInfo": {
        "signed": "2025-04-15T07:02:26Z",
        "validFrom": "2025-04-15T07:02:26Z",
        "validUntil": "2026-04-15T07:02:26Z"
      }
    }
  ]
}
```

The `attestation_server` must respond with a JSON-serialized `Vec<IssuableDocument>`. For example:

```http
HTTP/1.1 200 OK
Content-Type: application/json

[
  {
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

The JSON array may be empty (i.e. `[]`) to indicate that no attestations to be issued were found. In that case, the wallet will present a screen to the user explaining that there were no attestations found that can be issued.

### `issuance_server` configuration

For each attestation server, a configuration block like the following must be put in the configuration file of the `issuance_server`.

```toml
[disclosure_settings.degree]
# Either "hsm" or "software"
private_key_type = "hsm"

# In case of "hsm": label for the private key in the HSM
# In case of "software": a DER-serialized private key
private_key = "my_issuer_key"

# RP certificate
certificate = "MIJ..."

# Attributes that have to be disclosed for `degree` in DCQL format
[[disclosure_settings.degree.dcql_query.credentials]]
id = "my_credential"
format = "mso_mdoc"
meta = { doctype_value = "com.example.pid" }
claims = [
    { path = ["com.example.pid", "bsn"], intent_to_retain = true }
]

[disclosure_settings.degree.attestation_url_config]
# URL to the attestation server
base_url = "https://attestation_server.example.com"
trust_anchors = ["MIJ..."]

# A block like this has to be present for each attestation type that gets issued
[attestation_settings."com.example.degree"]
valid_days = 365
copy_count = 4
private_key_type = "software" # or "hsm", see above
private_key = "MIG..."        # DER-encoded private key, in case of "software"
certificate = "MIJ..."        # Issuer certificate

# Files containing SD-JWT Type Metadata documents for each attestation that will be issued
metadata = ["com.example.degree.json"]
```

Here, `degree` is an example of a freely choosable identifier that has to be present in the QR/UL that starts the session.

For the rest of the configuration parameters of the `issuance_server`, see the [`issuance_server` example configuration file](../../wallet_core/wallet_server/issuance_server/issuance_server.example.toml) and the [verifier documentation](../get-started/create-a-verifier.md#retrieve-disclosure-results).

### QR/UL

The wallet starts disclosure based issuance if it encounters a UL (or a QR with a UL within it) of a specific format. To create this UL, proceed as follows.

1. If your `issuance_server` is reachable on the internet by the wallet at `https://issuer.example.com`, create a URL of the following form:
    ```
    https://issuer.example.com/disclosure/degree/request_uri?session_type=same_device
    ```
    In which `degree` has to be the identifier mentioned above.
2. URL-encode the above URL.
3. Create the UL as follows (newlines only for readability purposes):
    ```
    https://app.example.com/deeplink/disclosure_based_issuance
      ?request_uri_method=post
      &client_id=disclosure_based_issuance.example.com
      &request_uri=https%3A%2F%2Fissuer.example.com...
    ```
    In which the `client_id` has to be the SAN DNS name from the RP `certificate`, and the `request_uri` is the URL-encoded URL from the previous step.

Next, place this UL on your website (within in a QR code in case of cross device flows).

## Sequence diagram

This diagram shows how we compose OpenID4VP with OpenID4VCI in the pre-authorized code flow to issue attestations based on the contents of other attestations.

The flow starts with the user scanning a QR code or tapping a UL that contains the URL to the disclosure based issuer. When the wallet contacts the issuer using this URL, it starts a disclosure session requesting some preconfigured set of attributes that will allow it to determine the attributes that it wants to issue. For example, suppose the issuer wishes to issue academic degrees based on the user's BSN, then it would preconfigure an Authorization Request that requests the BSN from the PID.

After the user has disclosed the requested attributes, the issuer sends those to an internal endpoint that responds with the attestations to be issued. In the earlier mentioned example, this would be a database containing academic degrees indexed by BSNs.

Finally, the issuer issues the attestations using OpenID4VCI in the pre-authorized code flow.

During OpenID4VCI, the issuer requires the wallet to include the keys of the attestations that it disclosed earlier in its Proof of Association (PoA) when it sends its Proofs of Possession (PoPs) for the keys of the attestation (copies). This enforces that the newly issued attestations are bound to the same WSCD as the one that disclosed the attestations in the first part of the protocol.

```{mermaid}
sequenceDiagram
    autonumber
    actor User
    participant App as Wallet
    participant Issuer as Disclosure based issuer
    participant IssuerDataProvider as Data Provider

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
