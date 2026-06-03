# Issuance with OpenID4VCI

We've implemented issuance with [OpenID4VCI 1.0][2] following the [High
Assurance Interoperability Profile (HAIP)][9], with attestation preview as a
custom addition. OpenID4VCI defines two grant flows and we use both across our
two issuance implementations.

We currently have two issuance implementations: the `pid_issuer`, a specialized
issuer specifically for Dutch [PID][1]s, which this page is about, and
`issuance_server`, a disclosure-based-issuance service that can issue all kinds
of things (which you can [read about here][8]). The `pid_issuer` uses the
authorization code flow described below; the `issuance_server` uses the
pre-authorized code flow.

## Pre-authorized code flow

In the [Pre-Authorized Code Flow][3] the issuer has already determined the
attributes to be issued out of band, binds them to a freshly generated
pre-authorized code in an issuance session, and hands that code to the wallet
inside an [OpenID4VCI Credential Offer][2]. The wallet sends the code to the
issuer's `/token` endpoint with
`grant_type=urn:ietf:params:oauth:grant-type:pre-authorized_code`; the issuer
loads the session keyed by the code to find the attributes to issue (this grant
carries no PKCE). The wallet then obtains a `c_nonce` from the issuer's [nonce
endpoint][14] and exchanges proofs of possession for attestations at the
`/credential` endpoint.

In our codebase, this flow is implemented by the `issuance_server` (the
disclosure-based-issuance service); see [Disclosure-based Issuance][8] for the
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
        Wallet->>CI: POST Credential Request<br/>(access_token, WUA) to /credential
        CI->>Wallet: Credential Response (attestation copies)
    end
```

## Authorization code flow (HAIP)

The [HAIP profile][9] mandates the regular Authorization Code Flow with a number
of additional requirements:

- A [Pushed Authorization Request (PAR)][10] is required; all authorization
  parameters are sent server-to-server and the browser only carries a
  `request_uri`;

- [PKCE][11] with `S256` is required;

- The wallet authenticates itself at both `/par` and `/token` with a Wallet
  Instance Attestation (WIA) carried in a `client_assertion` ([OAuth 2.0
  Attestation-Based Client Authentication][12]);

- Access tokens are sender-constrained with [DPoP][13]. The `/token` response
  has `token_type=DPoP`, and every subsequent request at the credential issuer
  carries both an `Authorization: DPoP <access_token>` header and a fresh DPoP
  proof JWT (which includes `ath`, the hash of the access token);

- OpenID4VCI 1.0 moves `c_nonce` out of the `/token` response into a dedicated
  [nonce endpoint][14] at the credential issuer;

- Issuance uses a single `/credential` endpoint that accepts a `proofs` array,
  carrying one Proof of Possession per attestation private key.

```{mermaid}
sequenceDiagram
    autonumber

    participant OS
    participant Wallet
    participant CI as Credential Issuer
    participant AS as Authorization Server

    Wallet-->>CI: discover OpenID4VCI metadata
    CI-->>Wallet: { credential_issuer,<br/>  credential_configurations_supported,<br/>  authorization_servers, nonce_endpoint,<br/>  credential_endpoint, ... }
    Wallet->>Wallet: discover Authorization Server

    Note over OS,AS: Authentication phase

    Wallet-->>AS: discover OAuth metadata
    AS-->>Wallet: { pushed_authorization_request_endpoint,<br/>  authorization_endpoint, token_endpoint, ... }
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
        Wallet-->>CI: GET metadata
        CI-->>Wallet: metadata
        Wallet->>CI: POST Credential Request<br/>(access_token, WUA) to /credential
        CI->>Wallet: Credential Response (attestation copies)
    end
```

## PID issuance

The NL Wallet requires at least the SD JWT format for PID attestations. The MSO
mDoc can be issued as well. The attestation type and paths to the `login` claim
and the `recovery_code` are dynamically configured. See the `pid_attributes`
field in the `wallet-config.json` for details.

In the `pid_issuer`, the Authorization Server and Credential Issuer roles are
combined: the wallet talks to a single `PID Issuer`, which hosts its own `/par`,
`/authorize`, `/token`, `/nonce` and `/credential` endpoints. Behind the scenes,
the `PID Issuer` delegates the actual user authentication to an upstream OpenID
Connect provider — in practice [RDO Max][6] acting as a DigiD broker.

For the internal structure of the `PID Issuer`, see
[PID Issuer architecture](../../development/pid-issuer-architecture.md).

### Two fully separate OAuth exchanges

The `PID Issuer` runs two independent OAuth 2.0 authorization-code exchanges
that share no parameters:

1. **Wallet ↔ PID Issuer** — the HAIP exchange described above. The wallet
   pushes a PAR, is sent to the `PID Issuer`'s `/authorize`, eventually receives
   an authorization code at its own `redirect_uri`, and exchanges it at the
   `PID Issuer`'s `/token`. The wallet's `client_id`, `redirect_uri`, `state`,
   PKCE pair, WIA (`client_assertion`) and DPoP all terminate at the
   `PID Issuer`.

2. **PID Issuer ↔ RDO Max** — a second, freshly minted exchange the
   `PID Issuer` starts while handling the wallet's `/authorize`. Every parameter
   here is the `PID Issuer`'s own: its DigiD `client_id`, its own `redirect_uri`
   (the `PID Issuer`'s `/digid/callback`), a random `state` (the
   `issuer_state`), its own PKCE pair, `scope=openid` and a fresh OIDC `nonce`.
   RDO Max redirects back to the `PID Issuer`'s callback — **never to the
   wallet**.

The `PID Issuer` terminates the upstream round-trip at its own callback,
generates its **own** authorization code, and only then redirects the browser
back to the wallet. RDO Max never redirects the browser to the wallet directly.
The two exchanges are linked solely by a server-side **state-bridge** entry,
keyed by the `issuer_state`, that remembers the wallet's original
`redirect_uri`, `state` and PKCE challenge (and the upstream PKCE verifier)
while the user is away at DigiD.

Because the upstream round-trip is terminated at the `PID Issuer`, the upstream
`/token` + `/userinfo` exchange (which yields the BSN) and the BRP attribute
lookup happen **in the `/digid/callback` handler**, before the wallet ever calls
`/token`. By the time the wallet exchanges its code at `/token`, the attributes
are already determined and stored in the issuance session, so `/token` only
verifies the wallet's PKCE `code_verifier` and issues the access token — there
is no upstream interaction at `/token`. If anything fails in the callback (BSN,
BRP, document build), the `PID Issuer` redirects the browser back to the
wallet's `redirect_uri` with an OAuth `error` response.

Parameter handling, **wallet ↔ PID Issuer** (everything terminates at the PID
Issuer):

| Parameter                  | Handling                                                                 |
| -------------------------- | ------------------------------------------------------------------------ |
| `client_id`                | validated against accepted wallet client ids; never forwarded            |
| `redirect_uri`             | wallet's universal link; the PID Issuer redirects here with its own code |
| `state`                    | remembered in the state bridge, echoed on the final wallet redirect      |
| `code_challenge`/`_method` | wallet's PKCE (`c1`, `S256`); verified at the PID Issuer's `/token`      |
| `scope`                    | the requested PID credential configuration                               |
| `client_assertion` (WIA)   | wallet ↔ PID Issuer only                                                |
| `DPoP` header              | wallet ↔ PID Issuer only                                                |

Parameter handling, **PID Issuer → RDO Max** (all freshly generated by the PID
Issuer, on both upstream `/authorize` and `/token`):

| Parameter                  | Value                                                              |
| -------------------------- | ------------------------------------------------------------------ |
| `client_id`                | the PID Issuer's DigiD client id                                   |
| `redirect_uri`             | the PID Issuer's own `/digid/callback`                             |
| `state`                    | random `issuer_state`; keys the state-bridge entry                 |
| `code_challenge`/`_method` | the PID Issuer's own PKCE pair (`c2`, `S256`)                      |
| `scope`                    | `openid`                                                           |
| `nonce` (OIDC)             | fresh random, required by nl-rdo-max                               |
| `code_verifier` (`/token`) | the PID Issuer's own verifier (`v2`); upstream client auth applied |

To keep the diagram focused on the OpenID4VCI / RDO Max delegation, the
wallet-side DPoP layer and the Wallet Instance Attestation (WIA,
`client_assertion`) are omitted below — both sit strictly between the wallet and
the `PID Issuer` and do not affect the delegation to RDO Max.

PKCE tracer values: the wallet uses `(v1, c1)` toward the `PID Issuer`; the
`PID Issuer` uses `(v2, c2)` toward RDO Max. Each verifier is checked only by
the party that issued the challenge.

```{mermaid}
sequenceDiagram
    autonumber

    participant OS
    participant Wallet
    participant PI as PID Issuer
    participant RDO as RDO Max

    %% ---- Optional issuer-initiated Credential Offer ----
    Note over OS,RDO: Credential Offer (optional, issuer-initiated)

    PI-->>OS: openid-credential-offer://?credential_offer_uri=<url><br/>(e.g. via QR code or same-device link)
    OS->>Wallet: open with offer URL
    Wallet->>PI: GET <credential_offer_uri>
    PI->>Wallet: { credential_issuer, credential_configuration_ids:<br/>  ["eu.europa.ec.eudi.pid_vc_sd_jwt"],<br/>  grants: { authorization_code: { issuer_state: "s_iss" } } }

    %% ---- Metadata discovery ----
    Wallet-->>PI: GET /.well-known/openid-credential-issuer
    PI-->>Wallet: { credential_issuer, credential_configurations_supported,<br/>  authorization_servers, nonce_endpoint,<br/>  credential_endpoint, ... }
    Wallet->>Wallet: pick AS from authorization_servers[]

    Note over OS,RDO: Authentication phase

    Wallet-->>PI: GET /.well-known/oauth-authorization-server
    PI-->>Wallet: { pushed_authorization_request_endpoint,<br/>  authorization_endpoint, token_endpoint,<br/>  require_pushed_authorization_requests: true }

    Wallet->>Wallet: generate PKCE_W (v1, c1 = S256(v1)),<br/>state=s1

    %% ---- PAR ----
    Wallet->>PI: POST /par
    note right of Wallet: response_type=code<br/>client_id=nl-wallet-app<br/>redirect_uri=<wallet_ul><br/>code_challenge=c1, code_challenge_method=S256<br/>state=s1<br/>scope=eu.europa.ec.eudi.pid_vc_sd_jwt<br/>issuer_state=s_iss   ← if from offer
    PI->>PI: store PAR keyed by request_uri
    PI->>Wallet: 201 { request_uri: "urn:...abc", expires_in: 60 }

    %% ---- Front-channel authorization ----
    Wallet->>OS: open browser → GET <PID_Issuer>/authorize?<br/>client_id=nl-wallet-app&request_uri=urn:...abc
    OS->>PI: GET /authorize?client_id=nl-wallet-app&request_uri=urn:...abc
    PI->>PI: consume PAR, validate client_id
    PI-->>RDO: GET /.well-known/openid-configuration<br/>(first upstream use, cached thereafter)
    RDO-->>PI: upstream OIDC metadata<br/>(authorization_endpoint, token_endpoint, userinfo_endpoint)
    PI->>PI: generate PKCE_P (v2, c2) and random issuer_state=s2,<br/>store state_bridge[s2] = {<br/>  wallet_redirect_uri=<wallet_ul>, wallet_state=s1,<br/>  wallet_code_challenge=c1, upstream_code_verifier=v2 }
    PI->>OS: 302 Location: <RDO>/authorize?<br/>response_type=code&client_id=pid-issuer-digid&<br/>redirect_uri=<PI>/digid/callback&<br/>code_challenge=c2&code_challenge_method=S256&<br/>state=s2&scope=openid&nonce=<random>
    OS->>RDO: GET /authorize?...
    note over OS,RDO: user authenticates via DigiD
    RDO->>OS: 302 Location: <PI>/digid/callback?<br/>code=up_code&state=s2
    OS->>PI: GET /digid/callback?code=up_code&state=s2

    %% ---- Upstream token + userinfo, BRP, mint issuer code ----
    PI->>PI: consume state_bridge[s2] → { wallet_*, v2 }
    PI->>RDO: POST /token<br/>(grant_type=authorization_code, code=up_code,<br/>redirect_uri=<PI>/digid/callback, code_verifier=v2,<br/>client_id=pid-issuer-digid, upstream client auth)
    RDO->>PI: { access_token: <upstream_at>, ... }
    PI->>RDO: GET /userinfo (Authorization: Bearer <upstream_at>)
    RDO->>PI: BSN (encrypted JWE)
    PI->>PI: Lookup attributes in BRP,<br/>build PID documents (SD-JWT + mdoc),<br/>mint authorization code=code1,<br/>store AuthCodeIssued session[code1] = {<br/>  documents, wallet_code_challenge=c1 }
    PI->>OS: 302 Location: <wallet_ul>?code=code1&state=s1
    OS->>Wallet: openWallet(code=code1, state=s1)
    Wallet->>Wallet: verify state == s1

    %% ---- Token exchange ----
    Wallet->>PI: POST /token
    note right of Wallet: grant_type=authorization_code<br/>code=code1<br/>redirect_uri=<wallet_ul><br/>code_verifier=v1      ← wallet's PKCE<br/>client_id=nl-wallet-app
    PI->>PI: load session[code1],<br/>verify S256(v1) == c1 (wallet PKCE),<br/>no upstream interaction
    PI->>Wallet: { access_token: <at>, token_type: "Bearer",<br/>  expires_in: 3600,<br/>  authorization_details: [{..., credential_identifiers:[...]}] }

    Note over OS,RDO: Issuance phase

    Wallet->>PI: POST /previews<br/>Authorization: Bearer <at>
    PI->>Wallet: previews

    loop for every credential
        Wallet->>PI: POST /nonce
        PI->>Wallet: { c_nonce: "n1" }
        Wallet-->>PI: GET /.well-known/openid-credential-issuer<br/>(usually cached)
        PI-->>Wallet: issuer metadata
        Wallet->>PI: POST /credential<br/>Authorization: Bearer <at>
        note right of Wallet: credential_configuration_id:<br/>  "eu.europa.ec.eudi.pid_vc_sd_jwt"<br/>proofs: { jwt: [<PoP_1>, <PoP_2>, ...] }
        PI->>Wallet: { credentials: [{ credential: "<sd-jwt-vc>" }, ...] }
    end
```

## Key generation and usage during issuance

### Wallet App

The wallet uses the Wallet Backend to generate attestation private keys and sign
the issuer's nonce with them. It does this by sending a `PerformIssuance` or
`PerformIssuanceWithWia` [instruction](./wallet-provider-instruction.md),
depending on whether or not a PID is being issued (which requires a WIA). Using
one of these instructions, the App requests the Wallet Backend to provide a WIA
and Proofs of Possession (PoPs) for the private keys by signing the `c_nonce`
from the issuer. The following sequence diagram depicts how this happens.

```{mermaid}
sequenceDiagram
    %% Force ordering by explicitly setting up participants
    participant wallet as Wallet Core (App)
    participant wallet_provider as Wallet Backend
    participant hsm as WB HSM
    participant db as WB Database

    wallet->>+wallet_provider: instruction: perform_issuance[_with_wia](c_nonce, key_count)
    wallet_provider->>wallet_provider: key_count++ if WIA is requested
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
    opt WIA requested
        wallet_provider ->> wallet_provider: generateWIAContent()
        wallet_provider ->>+ hsm: sign WIA content using wiaSigningPrivateKey
        hsm -->>- wallet_provider: WIA
    end
    opt More than 1 private key involved
        wallet_provider ->>+ hsm: sign PoA for attestationPrivateKeys and possibly WIA
        hsm -->>- wallet_provider: PoA
    end
    wallet_provider-->>-wallet: instruction response: PoPs, attestationPublicKeys, WIA (optional), PoA (optional)
```

<!-- References -->

[1]: https://eudi.dev/latest/annexes/annex-3/annex-3.01-pid-rulebook
[2]: https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html
[3]:
    https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#name-pre-authorized-code-flow
[6]: https://github.com/minvws/nl-rdo-max
[8]: disclosure-based-issuance
[9]:
    https://openid.github.io/OpenID4VC-HAIP/openid4vc-high-assurance-interoperability-profile-1_0-wg-draft.html
[10]: https://www.rfc-editor.org/rfc/rfc9126
[11]: https://www.rfc-editor.org/rfc/rfc7636
[12]:
    https://www.ietf.org/archive/id/draft-ietf-oauth-attestation-based-client-auth-05.html
[13]: https://www.rfc-editor.org/rfc/rfc9449
[14]:
    https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#name-nonce-endpoint
