# PID Issuer architecture

This page is a code-oriented companion to the protocol-level sequence diagram at
[Issuance with OpenID4VCI](../architecture/use-cases/issuance-with-openid4vci.md#pid-issuance).
That diagram treats the `PID Issuer` as a black box and shows what goes over the
wire between the wallet, the `PID Issuer` and RDO Max. This page opens that box:
which crates contribute which pieces, where state lives, and which traits are
the extension points for plugging in a different backend.

## Crate map

The `PID Issuer` process is assembled from three crates:

- **`wallet_core/lib/openid4vc`** — protocol types and traits, with no HTTP or
  storage baked in. Relevant pieces:
    - `authorization::PushedAuthorizationRequest`,
      `authorization::PushedAuthorizationResponse`,
      `authorization::AuthorizationRequest`, `token::TokenRequest`,
      `token::TokenResponse`, `credential::CredentialRequest(s)`,
      `credential::CredentialResponse(s)`.
    - `issuer::AttributeService` — the trait that produces the attributes to be
      issued given a `TokenRequest`.
    - `issuer::Issuer` — the protocol state machine: verifies token/credential
      requests, drives the session, calls into the `AttributeService`, generates
      access tokens, etc. `process_token_request` accepts an optional
      `UpstreamCodeVerifier` that it forwards verbatim to the
      `AttributeService` — the `Issuer` never inspects or interprets the value.
    - `par::ParStore`, `nonce::store::NonceStore`,
      `server_state::SessionStore<IssuanceData>`, `pkce::PkceFlowStore` —
      abstractions over where PAR entries, c_nonces, issuance sessions and
      upstream-PKCE bridging entries live. Default in-memory impls
      (`MemoryParStore`, `MemoryNonceStore`, `MemorySessionStore`,
      `MemoryPkceFlowStore`) ship alongside.

- **`wallet_core/lib/openid4vc_server`** — generic axum wiring for an OpenID4VCI
  issuer, knows nothing about DigiD or BRP:
    - `issuer::create_issuance_router` mounts the handlers on
      `/.well-known/openid-credential-issuer`,
      `/.well-known/oauth-authorization-server`, `/issuance/par`,
      `/issuance/authorize`, `/issuance/token`, `/issuance/nonce`,
      `/issuance/credential`, and `/issuance/credential_preview` (an extension
      we support on top of the spec).
    - `issuer::ApplicationState` bundles the `Issuer`, the `ParStore`, the
      `PkceFlowStore`, the optional upstream adapter, and the accepted wallet
      `client_id`s. Both `/authorize` (write) and `/token` (consume) consult the
      `PkceFlowStore` to bridge the decoupled wallet/upstream PKCE pairs.
    - `issuer::UpstreamAuthorizationAdapter` — the extension point the
      `/authorize` handler uses to resolve the upstream authorization endpoint
      and rewrite the wallet's `AuthorizationRequest` into one the upstream
      provider accepts. Letting the implementer own the full request mutation
      keeps non-standard quirks (e.g. nl-rdo-max requires a `nonce`) out of the
      generic handler.

- **`wallet_core/wallet_server/pid_issuer`** — the PID-specific concretions:
    - `pid::attributes::BrpPidAttributeService` — implements `AttributeService`
      by obtaining the BSN via DigiD and then looking up attributes in the BRP.
    - `pid::digid::DigidMetadataCache` — fetches and holds the upstream OIDC
      discovery document; shared between the adapter and `OpenIdClient`.
    - `pid::digid::DigidAuthorizationAdapter` — implements
      `UpstreamAuthorizationAdapter`; consults the `DigidMetadataCache` to
      resolve the upstream `authorization_endpoint`, rewrites the wallet's
      `client_id` to the DigiD `client_id`, sets `scope=openid`, and adds a
      fresh random `nonce` (required by nl-rdo-max).
    - `pid::digid::OpenIdClient` — drives the upstream token + userinfo exchange
      via `pid::userinfo::request_userinfo`: POST to RDO Max's `/token`, GET
      `/userinfo` as a signed-and-encrypted JWT, fetch the JWKS in parallel (via
      a per-call `pid::jwks::HttpJwksClient` wrapper), JWE-decrypt the payload
      and verify the JWS signature against the JWKS to extract the BSN.
    - `pid::brp::client::HttpBrpClient` (implements `BrpClient`) — queries the
      BRP for personal data by BSN.
    - `server::serve` wires all of the above into `create_issuance_router` and
      serves it.

## Component diagram

```{mermaid}
flowchart LR
    subgraph ovcs["openid4vc_server (HTTP wiring)"]
        direction TB
        Router["create_issuance_router<br/>axum handlers"]
        AppState["ApplicationState"]
        UpAd_trait["trait<br/>UpstreamAuthorizationAdapter"]
    end

    subgraph ovc["openid4vc (protocol types)"]
        direction TB
        Issuer["struct Issuer"]
        AS_trait["trait AttributeService"]
        Store_traits["trait ParStore<br/>trait PkceFlowStore<br/>trait SessionStore of IssuanceData<br/>trait NonceStore"]
        MemStores["MemoryParStore<br/>MemoryPkceFlowStore"]
    end

    subgraph pidi["pid_issuer (PID-specific)"]
        direction TB
        BrpAttr["BrpPidAttributeService"]
        DigidAdapter["DigidAuthorizationAdapter"]
        OpenIdClient["OpenIdClient"]
        DigidCache["DigidMetadataCache<br/>(holds OIDC metadata)"]
        BrpClient["HttpBrpClient"]
    end

    subgraph ext["external"]
        direction TB
        RDO[("RDO Max / DigiD")]
        BRP[("BRP")]
    end

    Router --> AppState
    AppState -->|holds| Issuer
    AppState -->|holds| MemStores
    AppState -->|holds| UpAd_trait
    Issuer -->|calls| AS_trait
    Issuer -->|reads/writes| Store_traits

    MemStores -.implements.-> Store_traits
    BrpAttr -.implements.-> AS_trait
    DigidAdapter -.implements.-> UpAd_trait

    BrpAttr -->|owns| OpenIdClient
    BrpAttr -->|owns| BrpClient
    OpenIdClient -. shares Arc .-> DigidCache
    DigidAdapter -. shares Arc .-> DigidCache

    DigidCache -->|GET /.well-known/<br/>openid-configuration| RDO
    OpenIdClient -->|POST /token,<br/>GET /userinfo,<br/>GET jwks_uri| RDO
    BrpClient -->|get_person_by_bsn| BRP
```

Two extension points are worth noting:

- `AttributeService` — swapping `BrpPidAttributeService` for a different
  implementation is how the generic `issuance_server` (disclosure-based
  issuance) reuses the same `openid4vc_server` handlers. The `/token` handler
  doesn't know what a BSN is.
- `UpstreamAuthorizationAdapter` — plugs in the upstream OIDC provider. Today
  the only implementor is `DigidAuthorizationAdapter`; a different IdP would get
  a sibling implementation here.

State lives behind the four `*Store` traits. The in-memory variants shown here
can all be replaced by stateful variants.

## Internal flow: PAR, authorize and token

This narrows the protocol diagram down to the lanes inside the `PID Issuer`
process across the three authorization-phase requests, so it's visible which
trait is consulted where and when the upstream DigiD round-trip happens relative
to the protocol state machine.

```{mermaid}
sequenceDiagram
    autonumber
    participant W as Wallet
    participant H as handler<br/>(openid4vc_server)
    participant R as DigidAuthorizationAdapter
    participant P as ParStore
    participant F as PkceFlowStore
    participant I as Issuer<br/>(openid4vc)
    participant A as BrpPidAttributeService
    participant O as OpenIdClient
    participant RDO as RDO Max
    participant BRP
    Note over W, F: POST /issuance/par
    W ->> H: POST /par (AuthorizationRequest)
    H ->> H: validate client_id against<br/>accepted_wallet_client_ids
    H ->> P: store(request_uri,<br/>authorization_request, expires_at)
    P ->> H: ok
    H ->> W: 201 PushedAuthorizationResponse<br/>(request_uri, expires_in)
    Note over W, F: GET /issuance/authorize
    W ->> H: GET /authorize?client_id,request_uri
    H ->> H: validate client_id
    H ->> P: consume(request_uri)
    P ->> H: AuthorizationRequest<br/>(including wallet code_challenge c1)
    H ->> H: generate upstream PKCE pair (v2, c2),<br/>rewrite code_challenge and<br/>code_challenge_method to (c2, S256)
    H ->> F: store(c1, v2)
    F ->> H: ok
    H ->> R: adapt(authorization_request)
    R ->> RDO: GET /.well-known/openid-configuration<br/>(cached after first call)
    RDO ->> R: OidcProviderMetadata
    R ->> R: rewrite client_id to DigiD client_id,<br/>set scope=openid,<br/>add fresh random nonce
    R ->> H: (authorization_endpoint,<br/>rewritten AuthorizationRequest)
    H ->> W: 302 Location: upstream /authorize?...
    Note over W, BRP: POST /issuance/token
    W ->> H: POST /token<br/>(TokenRequest with code_verifier v1)
    H ->> H: derive c1 = SHA256(v1)
    H ->> F: consume(c1)
    F ->> H: upstream code_verifier v2
    H ->> H: verify wallet code_verifier v1<br/>against c1 (PKCE check)
    H ->> I: process_token_request(token_request,<br/>upstream_code_verifier=Some(v2))
    I ->> A: attributes(token_request,<br/>upstream_code_verifier)
    A ->> O: bsn(code, code_verifier, redirect_uri)
    par fetch userinfo JWE
        O ->> RDO: POST {token_endpoint}<br/>(grant_type, code,<br/>redirect_uri, code_verifier,<br/>client_id)
        RDO ->> O: access_token (upstream)
        O ->> RDO: GET {userinfo_endpoint}<br/>Accept: application/jwt,<br/>Bearer (upstream_at)
        RDO ->> O: userinfo JWE (JWT)
    and fetch JWKS
        O ->> RDO: GET {jwks_uri}
        RDO ->> O: JwkSet
    end
    O ->> O: JWE-decrypt to JWS,<br/>verify JWS signature<br/>against JwkSet,<br/>extract BSN claim
    O ->> A: BSN
    A ->> BRP: get_person_by_bsn(bsn)
    BRP ->> A: person data
    A ->> A: build IssuableDocument<br/>(PID attestation,<br/>with recovery_code attribute)
    A ->> I: VecNonEmpty of IssuableDocument
    I ->> I: persist session,<br/>generate access_token and c_nonce
    I ->> H: TokenResponse
    H ->> W: TokenResponse
```

A few things the extended diagram makes visible:

- **Where the state stores are hit.** `ParStore::store` is called on `/par`,
  `ParStore::consume` is called on `/authorize`, `PkceFlowStore::store` is
  called on `/authorize`, `PkceFlowStore::consume` is called on `/token`, and
  the `SessionStore<IssuanceData>` load happens only inside the `Issuer` on
  `/token`.
- **Where the upstream-specific request mutation lives.** Everything the
  upstream provider needs that the wallet's request doesn't carry — DigiD's
  `client_id`, `scope=openid`, and the random `nonce` nl-rdo-max requires — is
  applied inside `DigidAuthorizationAdapter::adapt`, not in the
  `openid4vc_server` handler. The handler only validates the wallet's
  `client_id`, drives PAR consumption, and forwards what the adapter returns.
- **How the upstream `client_id` is set in each phase.** On `/authorize` the
  adapter rewrites the wallet's `client_id` to DigiD's. On `/token` there is no
  rewrite: `OpenIdClient` is constructed with the DigiD `client_id` and uses it
  when building its own upstream `TokenRequest`. The wallet's
  `TokenRequest.client_id` never reaches RDO Max.
- **The PKCE decoupling** is symmetric with the `client_id` rewrite but flipped
  in ownership, and lives entirely in `openid4vc_server`. At `/authorize` the
  handler generates its own pair `(v2, c2)`, substitutes the wallet's
  `code_challenge` with `c2`, and persists `v2` keyed by the wallet's `c1` in a
  dedicated `PkceFlowStore` (mirror of `ParStore`). At `/token` the handler
  recomputes `c1` from the wallet's `code_verifier`, looks up `v2`, verifies the
  wallet's verifier against `c1` (wallet-facing PKCE check), and delegates to
  `Issuer` — passing `v2` wrapped in an `UpstreamCodeVerifier` alongside the
  unmodified `TokenRequest`. The `Issuer` accepts the `UpstreamCodeVerifier`
  only as an opaque pass-through to the `AttributeService`; it never inspects or
  interprets the value. `BrpPidAttributeService` then unwraps `v2` from the
  `UpstreamCodeVerifier` (and reads `code` / `redirect_uri` from the
  `TokenRequest`) before calling `OpenIdClient::bsn`. The upstream PKCE check is
  then done by RDO Max. Result: wallet and RDO Max each see their own PKCE pair;
  neither sees the other's.

The protocol-level view of the same exchange — including which parameters the
`PID Issuer` rewrites on the way to RDO Max — is in
[Issuance with OpenID4VCI](../architecture/use-cases/issuance-with-openid4vci.md#pid-issuance).
