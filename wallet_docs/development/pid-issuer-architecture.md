# PID Issuer architecture

This page is a code-oriented companion to the protocol-level sequence diagram at
[Issuance with OpenID4VCI](../architecture/use-cases/issuance-with-openid4vci.md#pid-issuance).
That diagram treats the `PID Issuer` as a black box and shows what goes over the
wire between the wallet, the `PID Issuer` and RDO Max. This page opens that box:
which crates contribute which pieces, where state lives, and which traits are
the extension points for plugging in a different backend.

The key structural idea is that issuance is split into two phases, each with its
own component:

- an **Authorization Phase**, owned by `AuthorizingIssuer`, covering `/par`,
  `/authorize` and the upstream-IdP round-trip; and
- an **Issuance Phase**, owned by `Issuer`, covering `/token`, `/nonce`,
  `/credential` and the credential previews.

The Authorization Phase ends by writing an issuance session (keyed by an
authorization code) that the Issuance Phase then consumes. The pre-authorized
code flow (the `issuance_server`) skips the Authorization Phase entirely and
uses the bare `Issuer` directly.

## Crate map

The `PID Issuer` process is assembled from three crates:

- **`wallet_core/lib/openid4vc`** — protocol types and traits, with no HTTP or
  storage baked in. Relevant pieces:
    - `authorization::PushedAuthorizationRequest`,
      `authorization::PushedAuthorizationResponse`,
      `authorization::VciAuthorizationRequest` (with its `for_auth_code`
      constructor and the `OidcAuthorizationRequest` wrapper that adds the OIDC
      `nonce`), `token::TokenRequest`, `token::TokenResponse`,
      `credential::CredentialRequest(s)`, `credential::CredentialResponse(s)`.
    - `issuer::Issuer` — the **Issuance Phase** state machine: serves `/token`,
      `/nonce`, `/credential` and the previews. It holds the
      `SessionStore<IssuanceData>` and the nonce store. `process_token_request`
      loads the issuance session keyed by the code, verifies the wallet's PKCE
      according to the session's `Grant`, and issues the access token. It does
      **no** upstream interaction and knows nothing about DigiD or a BSN — by
      the time it runs, the issuables are already in the session.
    - `issuer::{AuthCodeIssued, Grant, IssuanceData}` — the issuance session
      data. `AuthCodeIssued` carries the `issuable_documents` plus a `Grant`:
      either `Grant::PreAuthorizedCode` (no PKCE) or
      `Grant::AuthorizationCode { wallet_code_challenge }` (the wallet's PKCE
      challenge, verified at `/token`). `Issuer::new_preauthorized_session`
      writes the pre-authorized variant directly; the auth-code variant is
      written by `AuthorizingIssuer::complete_authorization`.
    - `authorizing_issuer::AuthorizingIssuer` — the **Authorization Phase**
      wrapper around an `Issuer`. Owns the PAR store and an
      `AuthorizationCodeFlow` impl. Serves `/par` and `/authorize`, and exposes
      `complete_authorization`, which mints a fresh issuer-side authorization
      code, writes the `AuthCodeIssued` session (with
      `Grant::AuthorizationCode`), and builds the wallet-facing redirect URL.
      Deployments doing only the pre-authorized grant never construct one.
    - `authorization_code_flow::{AuthorizationCodeFlow, AuthorizeOutcome}` — the
      trait abstracting a single OAuth authorization-code grant at `/authorize`.
      `authorize()` returns either `AuthorizeOutcome::RedirectTo(url)` (send the
      user-agent to an external IdP) or `AuthorizeOutcome::IssuedCode(code)` (an
      issuer-minted code with no external round-trip). Any state that must
      survive between `/authorize` and the eventual callback is private to the
      impl. This is the seam where the upstream IdP is plugged in.
    - `par::ParStore` / `store::Store`, `nonce::store::NonceStore`,
      `server_state::SessionStore<IssuanceData>` — abstractions over where PAR
      entries, c_nonces and issuance sessions live. Default in-memory impls ship
      alongside. The upstream PKCE verifier is not held here; it travels in the
      flow's own state-bridge entry (see below).

- **`wallet_core/lib/openid4vc_server`** — generic axum wiring for an OpenID4VCI
  issuer, knows nothing about DigiD or BRP. It exposes two routers:
    - `issuer::create_issuance_router` mounts the **Issuance Phase** handlers:
      `/.well-known/openid-credential-issuer`,
      `/.well-known/oauth-authorization-server`, `/issuance/token`,
      `/issuance/nonce`, `/issuance/credential` (+ `batch_credential`, the
      `delete` reject routes) and `/issuance/credential_preview` (an extension
      we support on top of the spec). Backed by `IssuanceState { issuer }`. Both
      flows mount this: the pre-authorized `issuance_server` mounts it
      standalone, the auth-code `pid_issuer` mounts it alongside the
      authorization router.
    - `issuer::create_authorization_router` mounts the **Authorization Phase**
      handlers `/issuance/par` and `/issuance/authorize`. Backed by
      `AuthorizationState { authorizing_issuer }`. The `/authorize` handler just
      calls `AuthorizingIssuer::process_authorize` and 302-redirects to whatever
      URL it returns; it has no knowledge of PKCE bridging or the upstream IdP.
    - The upstream-IdP callback route is **not** in this crate. It is owned by
      the concrete `AuthorizationCodeFlow` impl and mounted by the binary.

- **`wallet_core/wallet_server/pid_issuer`** — the PID-specific concretions:
    - `pid::auth_code_flow::UpstreamOidcAuthorizationCodeFlow` — the concrete
      `AuthorizationCodeFlow`. It owns the DigiD client, the state-bridge store,
      the BRP client, the recovery-code HMAC key, the issuer's DigiD `client_id`
      and the issuer's own callback URL. Its `authorize()` generates the
      upstream PKCE pair and a random `issuer_state`, writes a
      `StateBridgeEntry` keyed by that `issuer_state`, and returns a
      `RedirectTo` the upstream `/authorize`. It also **owns the
      `/digid/callback` route** (via `callback_router`): the termination point
      for the upstream redirect, which consumes the bridge entry, obtains the
      BSN, looks up BRP attributes, builds the PID `IssuableDocument`s, and
      calls `AuthorizingIssuer::complete_authorization`.
    - `pid::digid::DigidMetadataCache` — fetches and holds the upstream OIDC
      discovery document.
    - `pid::digid::{DigidClient, HttpDigidClient}` — the trait + HTTP impl for
      the upstream exchange. `authorization_endpoint()` resolves the upstream
      `/authorize` URL (from the cache);
      `bsn(code, code_verifier, redirect_uri)` performs the upstream `/token` +
      `/userinfo` exchange (fetching the JWKS, JWE-decrypting and JWS-verifying
      the userinfo) to extract the BSN.
    - `pid::brp::client::HttpBrpClient` (implements `BrpClient`) — queries the
      BRP for personal data by BSN.
    - `server::serve` merges `create_issuance_router`,
      `create_authorization_router` and the flow's `callback_router`, and serves
      them.
    - `issuer_common::state_bridge_store::IssuerStateBridgeStore` — the store
      backing the flow's state bridge (Postgres or in-memory). It is generic
      over the entry type, serializing it to/from JSON, so the entry's shape
      stays private to the `AuthorizationCodeFlow` impl.

## Component diagram

```{mermaid}
flowchart LR
    subgraph ovcs["openid4vc_server (HTTP wiring)"]
        direction TB
        IssRouter["create_issuance_router<br/>(IssuanceState)"]
        AuthRouter["create_authorization_router<br/>(AuthorizationState)"]
    end

    subgraph ovc["openid4vc (protocol types)"]
        direction TB
        AuthIssuer["struct AuthorizingIssuer"]
        Issuer["struct Issuer"]
        AF_trait["trait AuthorizationCodeFlow<br/>(+ AuthorizeOutcome)"]
        Store_traits["trait ParStore / Store<br/>trait SessionStore of IssuanceData<br/>trait NonceStore"]
    end

    subgraph pidi["pid_issuer (PID-specific)"]
        direction TB
        Flow["UpstreamOidcAuthorizationCodeFlow<br/>(owns /digid/callback)"]
        DigidClient["HttpDigidClient"]
        DigidCache["DigidMetadataCache<br/>(holds OIDC metadata)"]
        BrpClient["HttpBrpClient"]
        Bridge["IssuerStateBridgeStore"]
    end

    subgraph ext["external"]
        direction TB
        RDO[("RDO Max / DigiD")]
        BRP[("BRP")]
    end

    AuthRouter --> AuthIssuer
    IssRouter --> Issuer
    AuthIssuer -->|wraps| Issuer
    AuthIssuer -->|holds| AF_trait
    AuthIssuer -->|reads/writes PAR| Store_traits
    Issuer -->|reads/writes sessions, nonces| Store_traits

    Flow -.implements.-> AF_trait
    Flow -->|owns| DigidClient
    Flow -->|owns| BrpClient
    Flow -->|reads/writes| Bridge
    Flow -->|complete_authorization| AuthIssuer
    DigidClient -. shares .-> DigidCache

    DigidCache -->|GET /.well-known/<br/>openid-configuration| RDO
    DigidClient -->|POST /token,<br/>GET /userinfo,<br/>GET jwks_uri| RDO
    BrpClient -->|get_person_by_bsn| BRP
```

The extension points worth noting:

- `AuthorizationCodeFlow` — plugs in the upstream IdP and its callback. Today
  the only implementor is `UpstreamOidcAuthorizationCodeFlow` (DigiD via RDO
  Max); a different IdP would get a sibling implementation that owns its own
  callback route. The generic `/authorize` handler doesn't know what DigiD is.
- The pre-authorized code flow reuses the bare `Issuer` (via
  `create_issuance_router`) without an `AuthorizingIssuer` at all. That is how
  the `issuance_server` (disclosure-based issuance) shares the Issuance Phase:
  it computes the issuables from the disclosed attributes and writes them with
  `Issuer::new_preauthorized_session`, then hands the wallet a Credential Offer
  carrying the pre-authorized code. The `/token` handler doesn't know what a BSN
  is.

State lives behind the `*Store` traits (PAR, sessions, nonces) plus the flow's
own `IssuerStateBridgeStore`. The in-memory variants can all be replaced by
stateful (Postgres) variants.

## Internal flow: PAR, authorize, callback and token

This narrows the protocol diagram down to the lanes inside the `PID Issuer`
process, so it's visible which component is consulted where and that the
upstream DigiD round-trip happens in the `/digid/callback` handler, before the
wallet's `/token`, not during it.

```{mermaid}
sequenceDiagram
    autonumber
    participant W as Wallet
    participant H as handler<br/>(openid4vc_server)
    participant AI as AuthorizingIssuer<br/>(openid4vc)
    participant F as UpstreamOidcAuthorizationCodeFlow<br/>(+ /digid/callback)
    participant B as IssuerStateBridgeStore
    participant I as Issuer<br/>(openid4vc)
    participant D as HttpDigidClient
    participant RDO as RDO Max
    participant BRP

    Note over W, BRP: POST /issuance/par
    W ->> H: POST /par (VciAuthorizationRequest)
    H ->> AI: process_pushed_authorization_request
    AI ->> AI: validate client_id,<br/>store(request_uri, request) in PAR store
    AI ->> W: 201 PushedAuthorizationResponse<br/>(request_uri, expires_in)

    Note over W, BRP: GET /issuance/authorize
    W ->> H: GET /authorize?client_id,request_uri
    H ->> AI: process_authorize(request_uri, client_id)
    AI ->> AI: validate client_id,<br/>consume(request_uri) from PAR store,<br/>check PAR client_id matches
    AI ->> F: authorize(authorization_request)
    F ->> F: generate upstream PKCE (v2, c2)<br/>and random issuer_state s2
    F ->> B: store(s2, { wallet_redirect_uri, wallet_state s1,<br/>  wallet_code_challenge c1, upstream_code_verifier v2 })
    F ->> D: authorization_endpoint()
    D ->> RDO: GET /.well-known/openid-configuration<br/>(cached after first call)
    RDO ->> D: OidcProviderMetadata
    D ->> F: upstream authorization_endpoint
    F ->> AI: AuthorizeOutcome::RedirectTo(upstream /authorize?…)
    AI ->> W: 302 Location: upstream /authorize?<br/>client_id=pid-issuer-digid, redirect_uri=<PI>/digid/callback,<br/>state=s2, code_challenge=c2, scope=openid, nonce=…

    Note over W, BRP: user authenticates via DigiD, then RDO Max redirects to /digid/callback

    Note over W, BRP: GET /digid/callback
    RDO ->> F: GET /digid/callback?code=up_code&state=s2
    F ->> B: consume(s2)
    B ->> F: { wallet_*, upstream_code_verifier v2 }
    F ->> D: bsn(up_code, v2, <PI>/digid/callback)
    par fetch userinfo JWE
        D ->> RDO: POST {token_endpoint}<br/>(grant_type, code=up_code,<br/>redirect_uri=<PI>/digid/callback,<br/>code_verifier=v2, client_id=pid-issuer-digid)
        RDO ->> D: access_token (upstream)
        D ->> RDO: GET {userinfo_endpoint}<br/>Accept: application/jwt, Bearer (upstream_at)
        RDO ->> D: userinfo JWE (JWT)
    and fetch JWKS
        D ->> RDO: GET {jwks_uri}
        RDO ->> D: JwkSet
    end
    D ->> D: JWE-decrypt to JWS,<br/>verify JWS against JwkSet,<br/>extract BSN claim
    D ->> F: BSN
    F ->> BRP: get_person_by_bsn(bsn)
    BRP ->> F: person data
    F ->> F: build IssuableDocuments<br/>(SD-JWT + mdoc, with recovery_code)
    F ->> AI: complete_authorization(documents,<br/>wallet_code_challenge c1, wallet_redirect_uri, wallet_state s1)
    AI ->> AI: mint authorization code code1
    AI ->> I: write_session(AuthCodeIssued[code1] =<br/>  { documents, Grant::AuthorizationCode { c1 } })
    AI ->> F: (code1, wallet redirect URL)
    F ->> W: 302 Location: <wallet_redirect_uri>?code=code1&state=s1

    Note over W, BRP: POST /issuance/token
    W ->> H: POST /token<br/>(code=code1, code_verifier=v1)
    H ->> I: process_token_request(token_request, dpop)
    I ->> I: load session[code1],<br/>verify S256(v1) == c1 (wallet PKCE),<br/>generate access_token + c_nonce<br/>(no upstream interaction)
    I ->> H: TokenResponse
    H ->> W: TokenResponse
```

A few things the extended diagram makes visible:

- **Where the upstream round-trip happens.** The upstream `/token` + `/userinfo`
  exchange and the BRP lookup are driven entirely by the `/digid/callback`
  handler, which runs _before_ the wallet's `/token`. By the time the wallet
  exchanges its code, the `Issuer` only verifies PKCE and reads the issuables
  from the session — it never talks to RDO Max or the BRP.
- **Where the state stores are hit.** `ParStore::store` on `/par`,
  `ParStore::consume` on `/authorize`; `IssuerStateBridgeStore::store` on
  `/authorize`, `IssuerStateBridgeStore::consume` on `/digid/callback`;
  `SessionStore<IssuanceData>` is _written_ on `/digid/callback` (inside
  `complete_authorization`) and _read_ on `/token` (and again on `/credential`).
- **The two OAuth exchanges are fully separate.** The upstream `client_id`,
  `redirect_uri` (the issuer's own `/digid/callback`), `state` (the random
  `issuer_state`) and PKCE pair `(v2, c2)` are all generated by the flow; none
  of the wallet's parameters are forwarded. The single link between them is the
  `StateBridgeEntry`, keyed by `issuer_state`, that the callback consumes to
  recover the wallet's `redirect_uri`, `state` and `code_challenge`.
- **The PKCE pairs never cross over.** The wallet's `(v1, c1)` is verified only
  by the `Issuer` at `/token`; the issuer's `(v2, c2)` is verified only by RDO
  Max at the upstream `/token`. The wallet's `code_challenge` never reaches RDO
  Max, and the issuer's never reaches the wallet.
- **Error handling.** If any callback step (BSN, BRP, document build) fails, the
  flow redirects the browser back to the wallet's `redirect_uri` with an OAuth
  `error` response (carrying the wallet's `state`), rather than surfacing a bare
  HTTP error.

The protocol-level view of the same exchange — including which parameters live
in which of the two OAuth exchanges — is in
[Issuance with OpenID4VCI](../architecture/use-cases/issuance-with-openid4vci.md#pid-issuance).
