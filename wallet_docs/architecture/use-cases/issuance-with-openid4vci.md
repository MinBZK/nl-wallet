# Issuance with OpenID4VCI

We've implemented issuance with [OpenID4VCI draft 13][2], with attestation
preview as a custom addition.

We currently (2025-12-11) have two issuance implementations: The `pid_issuer`, a
specialized issuer specifically for Dutch [PID][1]s, which this page is about,
and `issuance_server` a disclosure-based-issuance service that can issue all
kinds of things (which you can [read about here][8]).

## PID issuance

PID issuance is done by the `pid_issuer` which is a part of the `wallet_server`
crate. It is created to issue [PID][1]s specifically.

This diagram below shows how the `pid_issuer` uses [OpenID4VCI][2] in a
[Pre-Authorized Code Flow][3] to issue a [PID] to the wallet.

Using this protocol, the wallet starts a normal OpenID Connect session at an
`AuthServer` (which is a so-called [OpenID Provider][4]; in the case of the NL
Wallet, this usually means [DigiD][5] through [nl-rdo-max][6]), from which it
obtains an authorization code.

Next, the wallet uses this code to start the OpenID4VCI issuance protocol in the
pre-authorized code flow with a `wallet_server` component called `pid_issuer`.
The `pid_issuer` finishes the OpenID Connect session with the `AuthServer` to
discover the identity of the wallet user, allowing it to finish issuance.

In the diagram below we introduce an actor called the `PidAttributeService`,
whose responsibility it is to produce the attributes to be issued when given a
valid pre-authorized code.

In the case of PID issuance, the `pid_issuer` can do this by finishing the
OpenID Connect session that the wallet started. (The `PidAttributeService` is a
part of the `pid_issuer` in the `wallet_server` crate, as opposed to a separate
HTTP server; we include it here as separate actor to clearly visualize separate
responsibilities.)

The protocol works as follows:

- The wallet starts an OpenID Connect session at the `AuthServer` by sending it
  an Authorization Request, receiving an authorization code from the
  `AuthServer` in response;

- Using the received authorization code, the wallet starts OpenID4VCI issuance
  in a so-called pre-authorized code flow by sending a `POST` request with the
  previously obtained code as a pre-authorized code in a Token Request to the
  `pid_issuer`;

- The `pid_issuer` feeds the Token Request with the pre-authorized code to its
  `PidAttributeService` component. The `PidAttributeService` `POST`'s the Token
  Request to the `AuthServer`, transforming only the pre-authorized code in it
  to a normal authorization code but keeping the other parameters (such as the
  `state` and the PKCE `code_verifier`) in the Token Request as-is, thereby
  continuing the OpenID Connect session that the wallet previously started, and
  obtaining an `access_token`;

- Using the resulting `access_token`, the `PidAttributeService` invokes the
  `/userinfo` endpoint of the `AuthServer` to retrieve the BSN, with which it
  does a query to the [BRP][7], resulting in the attributes to be issued;

- The `pid_issuer` then generates the `c_nonce` and an `access_token` of its
  own, and a preview of the attestations as a custom addition to the OpenID4VCI
  protocol, all of which it returns to the wallet;

- With the `access_token` and a valid set of proofs of possession (signatures
  over the `c_nonce` validating against the public keys that the wallet wants to
  have in its PID), the wallet can access the `batch_credential` endpoint of the
  `pid_issuer` to obtain the attestations.

Notice that from the perspective of the `AuthServer`, the mobile operating
system (abreviated as "OS" in the diagram) acts as the User Agent in an OpenID
Connect session, and the `pid_issuer`'s `PidAttributeService` acts as the OpenID
Client. The former starts the session by navigating to the `AuthServer` with an
Authorization Request, and the latter resumes the session with Token and User
Info Requests.

The NL Wallet requires at least the SD JWT format for PID attestations. The MSO
mDoc can be issued as well. The attestation type and paths to the `login` claim
and the `recovery_code` are dynamically configured. See the `pid_attributes`
field in the `wallet-config.json` for details.

```{mermaid}
sequenceDiagram
    autonumber

    actor User
    participant OS
    participant Wallet
    participant WP as Wallet Backend
    participant WalletServer as PidIssuer
    participant PidAttributeService
    participant AuthServer

    User->>+Wallet: click "issue PID"
    Wallet->>-OS: navigate to AuthServer/authorize?redirect_uri=...
    OS->>+AuthServer: GET /authorize?redirect_uri=...
    note over User, AuthServer: authenticate user with DigiD app
    AuthServer->>AuthServer: generate & store code
    AuthServer->>-OS: GET universal_link?code=...
    OS->>Wallet: openWallet(code)
    activate Wallet
        Wallet->>+WalletServer: POST /token(pre-authorized_code)
        WalletServer->>+PidAttributeService: getAttributes(pre-authorized_code)
        PidAttributeService->>+AuthServer: POST /token(code)
        AuthServer->>AuthServer: lookup(code)
        AuthServer->>-PidAttributeService: access_token
        PidAttributeService->>+AuthServer: GET /userinfo(access_token)
        AuthServer->>-PidAttributeService: claims(BSN)
        PidAttributeService->>PidAttributeService: obtain attributes from BRP
        PidAttributeService->>-WalletServer: attributes
        WalletServer->>WalletServer: generate c_nonce, access_token
        WalletServer->>-Wallet: access_token, c_nonce, attestation_previews
        Wallet->>+User: Show attributes, ask consent
    deactivate Wallet
    User->>-Wallet: approve with PIN
    activate Wallet
        Wallet ->>+ WP: request PoPs with nonce<br/>(PerformIssuanceWithWia instruction)
        WP ->>- Wallet: Return WIA and Signed PoP and PoA
        Wallet->>+WalletServer: POST /batch_credential(access_token, PoPs)
        note over Wallet: WIA and PoA are included here
        WalletServer->>WalletServer: verify proofs,  WIA and PoA
        WalletServer->>-Wallet: attestations
    deactivate Wallet
```

## OpenID4VCI 1.0 + HAIP authorization code flow

The diagram above reflects the current implementation using [OpenID4VCI draft
13][2] in the [Pre-Authorized Code Flow][3]. We are moving towards [OpenID4VCI
1.0][2] in combination with the [High Assurance Interoperability Profile
(HAIP)][9], which mandates the regular Authorization Code Flow with a number of
additional requirements:

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

- The draft-13 `/batch_credential` endpoint is replaced by a single
  `/credential` endpoint that accepts a `proofs` array, carrying one Proof of
  Possession per attestation private key.

In the `pid_issuer`, the Authorization Server and Credential Issuer roles are
combined: the wallet talks to a single `PID Issuer`, which hosts its own `/par`,
`/authorize`, `/token`, `/nonce` and `/credential` endpoints. Behind the scenes,
the `PID Issuer` still delegates the actual user authentication to an upstream
OpenID Connect provider — in practice [RDO Max][6] acting as a DigiD broker.

For the internal structure of the `PID Issuer` — which crates contribute the
handlers, the `AttributeService` and `UpstreamAuthorizationEndpointResolver`
seams, where state lives, and how a single `/token` request flows through the
components — see
[PID Issuer architecture](../../development/pid-issuer-architecture.md).

### Delegation to RDO Max: what is decoupled, what is passed through

Conceptually the `PID Issuer` runs its own Authorization Server in front of RDO
Max. Most OAuth parameters are therefore _decoupled_: the wallet's values are
terminated at the `PID Issuer`, which substitutes its own for the upstream
server. A few parameters are _shared_ so that RDO Max can redirect the browser
straight back to the wallet without any intermediary redirects to the PID
Issuer's domain, and lets RDO Max's error/cancel responses reach the wallet
directly.

Parameter handling on **upstream `/authorize`**:

| Parameter                         | Handling                                                                |
| --------------------------------- | ----------------------------------------------------------------------- |
| `client_id`                       | **decoupled** — wallet's client id → PID Issuer's DigiD client id       |
| `redirect_uri`                    | **shared** — wallet's universal link, passed through                    |
| `code_challenge`/`_method`        | **decoupled** — PID Issuer generates its own PKCE pair for the upstream |
| `state`                           | **shared** — so the wallet can verify it when the code comes back       |
| `response_type=code`              | **shared**                                                              |
| `scope` / `authorization_details` | **terminated** — replaced with `openid`                                 |
| `nonce` (OIDC)                    | PID Issuer generates its own for the upstream session                   |
| `client_assertion` (WIA)          | **terminated** — wallet ↔ PID Issuer only                              |

Parameter handling on **upstream `/token`**:

| Parameter                       | Handling                                                               |
| ------------------------------- | ---------------------------------------------------------------------- |
| `client_id`                     | **decoupled** — rewritten as above                                     |
| `code`                          | **shared** — the upstream code flows wallet → PID Issuer → RDO Max     |
| `redirect_uri`                  | **shared**                                                             |
| `code_verifier`                 | **decoupled** — wallet sends its verifier; PID Issuer swaps in its own |
| `grant_type=authorization_code` | **shared**                                                             |
| `client_assertion` (WIA)        | **terminated**                                                         |
| `DPoP` header                   | **terminated** — RDO Max is not DPoP-aware; upstream client auth used  |

To keep the diagram focused on the OpenID4VCI / RDO Max delegation, the
wallet-side DPoP layer and the Wallet Instance Attestation (WIA,
`client_assertion`) are omitted below — both sit strictly between the wallet and
the `PID Issuer` and do not affect the delegation to RDO Max.

PKCE tracer values: wallet uses `(v1, c1)`, the `PID Issuer` uses `(v2, c2)` on
the upstream server.

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
    note right of Wallet: response_type=code<br/>client_id=nl-wallet-app<br/>redirect_uri=walletdebuginteraction://deeplink<br/>code_challenge=c1, code_challenge_method=S256<br/>state=s1<br/>authorization_details=[{type:"openid_credential",<br/>  credential_configuration_id:"eu.europa.ec.eudi.pid_vc_sd_jwt"}]<br/>issuer_state=s_iss   ← if from offer
    PI->>PI: bind session S,<br/>store PAR keyed by request_uri
    PI->>Wallet: 201 { request_uri: "urn:...abc", expires_in: 60 }

    %% ---- Front-channel authorization ----
    Wallet->>OS: open browser → GET <PI>/authorize?<br/>client_id=nl-wallet-app&request_uri=urn:...abc
    OS->>PI: GET /authorize?client_id=nl-wallet-app&request_uri=urn:...abc
    PI-->>RDO: GET /.well-known/openid-configuration<br/>(first upstream use, cached thereafter)
    RDO-->>PI: upstream OIDC metadata<br/>(authorization_endpoint, token_endpoint, userinfo_endpoint)
    PI->>PI: generate PKCE_P (v2, c2)<br/>and upstream nonce, attach to session S
    note right of PI: rewrite on the way to RDO Max:<br/>client_id       nl-wallet-app  →  pid-issuer-digid<br/>code_challenge  c1             →  c2<br/>scope/authz_details  terminated → scope=openid<br/>redirect_uri    (passed through)<br/>state           (passed through)<br/>response_type   (passed through)
    PI->>OS: 302 Location: <RDO>/authorize?<br/>response_type=code&client_id=pid-issuer-digid&<br/>redirect_uri=walletdebuginteraction://deeplink&<br/>code_challenge=c2&code_challenge_method=S256&<br/>state=s1&scope=openid
    OS->>RDO: GET /authorize?...
    note over OS,RDO: user authenticates via DigiD
    RDO->>OS: 302 Location: walletdebuginteraction://deeplink?<br/>code=code1&state=s1
    OS->>Wallet: openWallet(code=code1, state=s1)
    Wallet->>Wallet: verify state == s1

    %% ---- Token exchange ----
    Wallet->>PI: POST /token
    note right of Wallet: grant_type=authorization_code<br/>code=code1<br/>redirect_uri=walletdebuginteraction://deeplink<br/>code_verifier=v1      ← wallet's PKCE<br/>client_id=nl-wallet-app
    PI->>PI: verify code_verifier v1 against c1,<br/>load session S → v2
    note right of PI: rewrite on the way to RDO Max:<br/>client_id      nl-wallet-app  →  pid-issuer-digid<br/>code_verifier  v1             →  v2<br/>code, redirect_uri, grant_type  (passed through),<br/>upstream client auth applied
    PI->>RDO: POST /token<br/>grant_type=authorization_code,<br/>code=code1,<br/>redirect_uri=walletdebuginteraction://deeplink,<br/>code_verifier=v2, client_id=pid-issuer-digid<br/>(with upstream client auth)
    RDO->>PI: { access_token: <upstream_at>, ... }
    PI->>RDO: GET /userinfo (Authorization: Bearer <upstream_at>)
    RDO->>PI: BSN (encrypted JWE)
    PI->>PI: Lookup attributes in BRP
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
[4]: https://openid.net/developers/how-connect-works
[5]: https://www.logius.nl/onze-dienstverlening/toegang/digid
[6]: https://github.com/minvws/nl-rdo-max
[7]: https://www.rvig.nl/basisregistratie-personen
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
