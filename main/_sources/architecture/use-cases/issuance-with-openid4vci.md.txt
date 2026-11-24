# Issuance with OpenID4VCI

We've implemented issuance with [OpenID4VCI 1.0][2] following the [High
Assurance Interoperability Profile (HAIP)][5], with attestation preview as a
custom addition. OpenID4VCI defines two grant flows and we use both across our
two issuance implementations.

We currently have two issuance implementations: the `pid_issuer`, a specialized
issuer specifically for Dutch [PID][1]s, which this page is about, and
`issuance_server`, a disclosure-based-issuance service that can issue all kinds
of things (which you can [read about here][4]). The `pid_issuer` uses the
authorization code flow described below; the `issuance_server` uses the
pre-authorized code flow.

## Pre-authorized code flow

In the [Pre-Authorized Code Flow][3] the issuer has already determined the
attributes to be issued out of band, binds them to a freshly generated
pre-authorized code in an issuance session, and hands that code to the wallet
inside an [OpenID4VCI Credential Offer][2]. The wallet sends the code to the
issuer's `/token` endpoint with
`grant_type=urn:ietf:params:oauth:grant-type:pre-authorized_code`,
authenticating itself with a Wallet Instance Attestation (WIA) sent in HTTP
headers ([OAuth 2.0 Attestation-Based Client Authentication][8]); the issuer
verifies the WIA and loads the session keyed by the code to find the attributes
to issue (this grant carries no PKCE). The wallet then obtains a `c_nonce` from
the issuer's [nonce endpoint][10] and exchanges proofs of possession for
attestations at the `/credential` endpoint.

In our codebase, this flow is implemented by the `issuance_server` (the
disclosure-based-issuance service); see [Disclosure-based Issuance][4] for the
end-to-end picture, including how the disclosed attributes are turned into
attestations via the _attestation server_. The `pid_issuer` does not accept the
pre-authorized code grant — it only accepts the authorization code flow
described below.

```{mermaid}
sequenceDiagram
    autonumber

    participant OS
    participant Wallet
    participant CI as Credential Issuer
    participant AS as Authorization Server

    Note over OS,AS: Credential Offer (out of band)

    CI-->>OS: openid-credential-offer://?credential_offer_uri=<url><br/>(e.g. via QR code or universal link)
    OS->>Wallet: open with offer URL
    Wallet->>CI: GET <credential_offer_uri>
    CI->>Wallet: Credential Offer<br/>{ credential_issuer, credential_configuration_ids,<br/>  grants: { pre-authorized_code:<br/>    { pre-authorized_code, ... } } }

    Note over OS,AS: Discovery

    Wallet-->>CI: discover OpenID4VCI metadata
    CI-->>Wallet: issuer metadata
    Wallet->>Wallet: discover Authorization Server
    Wallet-->>AS: discover OAuth metadata
    AS-->>Wallet: oauth metadata

    Note over OS,AS: Token exchange

    Wallet->>AS: POST Token Request<br/>(grant_type=...:pre-authorized_code,<br/>pre-authorized_code, WIA)
    AS->>Wallet: Token Response (access_token)

    Note over OS,AS: Issuance phase

    Wallet->>CI: POST /previews(access_token)
    CI->>Wallet: previews

    loop for every credential
        Wallet->>CI: POST Nonce Request
        CI->>Wallet: c_nonce
        Wallet-->>CI: GET metadata
        CI-->>Wallet: metadata
        Wallet->>CI: POST Credential Request<br/>(access_token) to /credential
        CI->>Wallet: Credential Response (attestation copies)
    end
```

## Authorization code flow (HAIP)

The [HAIP profile][5] mandates the regular Authorization Code Flow with a number
of additional requirements:

- A [Pushed Authorization Request (PAR)][6] is required; all authorization
  parameters are sent server-to-server and the browser only carries a
  `request_uri`;

- [PKCE][7] with `S256` is required;

- The wallet authenticates itself at both `/par` and `/token` with a Wallet
  Instance Attestation (WIA), carried in `OAuth-Client-Attestation` and
  `OAuth-Client-Attestation-PoP` headers ([OAuth 2.0 Attestation-Based Client
  Authentication][8]);

- Access tokens are sender-constrained with [DPoP][9]. The `/token` response has
  `token_type=DPoP`, and every subsequent request at the credential issuer
  carries both an `Authorization: DPoP <access_token>` header and a fresh DPoP
  proof JWT (which includes `ath`, the hash of the access token);

- Issuance uses a single `/credential` endpoint that accepts a `proofs` array,
  carrying one Proof of Possession per attestation private key.

```{mermaid}
sequenceDiagram
    autonumber

    participant OS
    participant Wallet
    participant CI as Credential Issuer
    participant AS as Authorization Server

    CI-->>OS: openid-credential-offer://?credential_offer_uri=<url><br/>(e.g. via QR code or universal link)
    OS->>Wallet: open with offer URL
    Wallet->>CI: GET <credential_offer_uri>
    CI->>Wallet: Credential Offer<br/>{ credential_issuer, credential_configuration_ids,<br/>  grants: { authorization_code:<br/>    { ... } } }

    Note over OS,AS: Discovery phase

    Wallet-->>CI: discover OpenID4VCI metadata
    CI-->>Wallet: { credential_issuer,<br/>  credential_configurations_supported,<br/>  authorization_servers, nonce_endpoint,<br/>  credential_endpoint, ... }

    Wallet->>Wallet: discover Authorization Server

    Wallet-->>AS: discover OAuth metadata
    AS-->>Wallet: { pushed_authorization_request_endpoint,<br/>  authorization_endpoint, token_endpoint, ... }

    Note over OS,AS: Authentication phase

    Wallet->>AS: POST PAR (WIA)
    AS->>Wallet: request_uri
    Wallet->>OS: open browser (URL)
    OS->>AS: (browser) GET Authorization Request (request_uri)
    AS->>OS: Authorization Response (code)
    OS->>Wallet: openWallet(code)
    Wallet->>AS: POST Token Request (code, WIA)
    AS->>Wallet: Token Response (access_token)

    Note over OS,AS: Issuance phase

    Wallet->>CI: POST /previews(access_token)
    CI->>Wallet: previews

    loop for every credential
        Wallet->>CI: POST Nonce Request
        CI->>Wallet: c_nonce
        Wallet-->>CI: GET /metadata
        CI-->>Wallet: metadata
        Wallet->>CI: POST Credential Request<br/>(access_token) to /credential
        CI->>Wallet: Credential Response (attestation copies)
    end
```

## PID issuance

The NL Wallet requires at least the SD JWT format for PID attestations. The MSO
mDoc format can be issued as well. The attestation type and paths to the `login`
claim and the `recovery_code` are dynamically configured. See the
`pid_attributes` field in the `wallet-config.json` for details.

In the `pid_issuer`, the Authorization Server and Credential Issuer roles are
combined: the wallet talks to a single `PID Issuer`, which hosts its own `/par`,
`/authorize`, `/token`, `/nonce` and `/credential` endpoints.

The `PID Issuer` is a simple OAuth 2.0 Authorization Code Flow issuer that
follows the HAIP exchange described above.

## Timeouts and expiry

Issuance spans several hops — an out-of-band offer, one or two OAuth
round-trips, and a multi-step issuance phase — and each hop is bounded by its
own, independent clock. Where a timeout becomes visible, and whether it can be
surfaced to the wallet at all, depends on _which_ clock expired and _which_ flow
is in play.

### The clocks

| Clock                       | Flow      | Default | Where set                          | Bounds                                                                                         |
| --------------------------- | --------- | ------- | ---------------------------------- | ---------------------------------------------------------------------------------------------- |
| PAR entry TTL               | auth-code | 60 s    | `PAR_TTL`                          | time between `/par` and the front-channel `/authorize`                                         |
| State-bridge entry TTL      | auth-code | 30 min  | `STATE_BRIDGE_ENTRY_TTL`           | the whole user-agent round-trip for the login or consent page                                  |
| State-bridge delete leeway  | auth-code | 24 h    | `STATE_BRIDGE_DELETE_LEEWAY`       | extra grace after the TTL in which an _expired_ entry stays recoverable                        |
| Issuance session inactivity | both      | 30 min  | `SessionStoreTimeouts::expiration` | inactivity between `/token`, previews and `/credential`; also a pre-authorized code's lifetime |

The issuance session also has deletion timeouts (`successful_deletion`, 5 min;
`failed_deletion`, 4 h) that govern _cleanup_ after the session is done or
expired, mirroring the state bridge's split between expiration and deletion.

### Where an expiry surfaces

The decisive question is whether the expiry is hit on a **wallet→issuer API
call** (which can carry a structured error back to the wallet) or **during a
front-channel browser redirect** (where the only way back to the wallet is the
`redirect_uri`, which may no longer be recoverable).

| Failure mode                              | Hit on        | Wire signal                      | Wallet result                         |
| ----------------------------------------- | ------------- | -------------------------------- | ------------------------------------- |
| Pre-authorized code reused / session gone | `/token` call | `invalid_grant` (RFC 6749 §5.2)  | "QR code no longer valid" screen      |
| Issuance session expired mid-issuance     | `/credential` | `invalid_token` (RFC 6750, 401)  | "session expired" screen              |
| State-bridge entry expired, within leeway | browser       | OAuth `invalid_request` redirect | redirect-uri error                    |
| State-bridge entry deleted (after leeway) | browser       | plain-text 400                   | **dead-end** — nothing reaches wallet |

- **Pre-authorized code, reused or expired** (`issuance_server`). The wallet
  POSTs the code to `/token` but the session is gone or already `Done`. This
  maps to the spec's `invalid_grant`; the wallet recognises a pre-authorized
  `invalid_grant` and shows a "QR code no longer valid" screen. It is a plain
  API call, so the error reaches the wallet cleanly.

- **Issuance session expired mid-issuance** (both flows). The holder dwells on a
  preview or the PIN screen past the inactivity timeout and the session is
  cleaned up. The next `/credential` call presents an access token whose session
  no longer exists, which maps to `invalid_token`; the wallet shows a "session
  expired" screen. Also an API call, so it surfaces cleanly.

- **Auth-code browser dead-end** (both `pid_issuer` and `acf_demo_issuer`).
  During the front-channel round-trip — login page for the PID issuer, the local
  consent page for the acf-demo issuer — the state-bridge entry expires. The
  callback then arrives carrying only the opaque `state` (the bridge key); the
  wallet's `redirect_uri` lived _solely_ inside that entry. Two sub-cases:
    - **Within the delete leeway**, the entry is expired-but-still-present, so
      the callback recovers the `redirect_uri` and redirects the browser back to
      the wallet with an OAuth error (`invalid_request`). The wallet surfaces
      this as a redirect-uri error rather than a dead-end. This is exactly what
      the leeway buys.
    - **After the leeway**, the entry has been deleted, nothing links back to
      the wallet, and the callback dead-ends as a plain-text `400` in the
      browser.

### Relating the clocks

- The **delete leeway is deliberately far longer than the TTL** (24 h vs 30
  min), so a user who steps away mid-flow and returns much later still gets a
  graceful error redirect instead of a dead-end. The bridge entry holds no
  personal data (only the wallet's `redirect_uri`, `state` and PKCE challenge),
  so retaining it that long is cheap.

## Key generation and usage during issuance

### Wallet App

The wallet uses the Wallet Backend to generate attestation private keys and sign
the issuer's nonce with them. It does this by sending a `PerformIssuance`
[instruction](./wallet-provider-instruction.md), which returns one Proof of
Possession (PoP) per generated key by signing the `c_nonce` from the issuer.

For PID issuance, a WIA is additionally required. The wallet sends an
[`IssueWia`](./wallet-provider-instruction.md) instruction, which returns a
fresh WIA (signed with the WB's certificate chain) together with a disclosure
PoP that binds the WIA to the current request.

The following sequence diagram depicts how this happens.

```{mermaid}
sequenceDiagram
    %% Force ordering by explicitly setting up participants
    participant wallet as Wallet Core (App)
    participant wallet_provider as Wallet Backend
    participant hsm as WB HSM
    participant db as WB Database

    wallet->>+wallet_provider: instruction: perform_issuance(c_nonce, key_count)
    wallet_provider ->>+ hsm: generateECDSAPrivateKeys(key_count)
    hsm ->> hsm: generate ECDSA private keys<br/>encrypt each private key with attestationWrappingKey
    hsm -->>- wallet_provider: encryptedECDSAPrivateKeys, ECDSAPublicKeys
    wallet_provider ->>+ db : storeAttestationKeys(encryptedAttestationPrivateKeys, attestationPublicKeys)
    db -->>- wallet_provider: OK
    loop for every encryptedAttestationPrivateKey
        wallet_provider ->>+ hsm: sign(encryptedAttestationPrivateKey, c_nonce)
        hsm ->> hsm: Decrypt encryptedAttestationPrivateKey with attestationWrappingKey<br/>sign c_nonce with decrypted key
        hsm -->>- wallet_provider: PoP
    end
    wallet_provider-->>-wallet: instruction response: keyIdentifiers, PoPs

    opt WIA required (PID issuance)
        wallet->>+wallet_provider: instruction: issue_wia(nonce, aud)
        wallet_provider ->>+ hsm: generateECDSAPrivateKey()
        hsm ->> hsm: generate WIA signing key<br/>encrypt with attestationWrappingKey
        hsm -->>- wallet_provider: encryptedWIAPrivateKey, WIAPublicKey
        wallet_provider ->> wallet_provider: buildWIAContent(walletMetadata, statusClaim)
        wallet_provider ->>+ hsm: sign WIA content with wiaSigningKey (x5c)
        hsm -->>- wallet_provider: WIA JWT
        wallet_provider ->>+ hsm: sign disclosure PoP with wiaSigningKey
        hsm -->>- wallet_provider: disclosure PoP
        wallet_provider ->>+ db: storeWIAKey(encryptedWIAPrivateKey)
        db -->>- wallet_provider: OK
        wallet_provider-->>-wallet: instruction response: WiaDisclosure (WIA JWT + disclosure PoP)
    end
```

<!-- References -->

[1]: https://eudi.dev/latest/annexes/annex-3/annex-3.01-pid-rulebook
[2]: https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html
[3]:
    https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#name-pre-authorized-code-flow
[4]: disclosure-based-issuance
[5]:
    https://openid.github.io/OpenID4VC-HAIP/openid4vc-high-assurance-interoperability-profile-1_0-wg-draft.html
[6]: https://www.rfc-editor.org/rfc/rfc9126
[7]: https://www.rfc-editor.org/rfc/rfc7636
[8]:
    https://datatracker.ietf.org/doc/html/draft-ietf-oauth-attestation-based-client-auth-09
[9]: https://www.rfc-editor.org/rfc/rfc9449
[10]:
    https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#name-nonce-endpoint
