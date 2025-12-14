# Issuance with OpenID4VCI

We've implemented issuance with [OpenID4VCI draft 13][2], with attestation
preview as a custom addition.

We currently (2025-12-11) have two issuance implementations: The `pid_issuer`,
a specialized issuer specifically for [PID][1]s, which this document is about,
and `issuance_server` a reference disclosure-based-issuance service that can
issue all kinds of things (which you can [read about here][8]).

## PID issuance

PID issuance is done by the `pid_issuer` which is a part of the
`wallet_server` crate. It is created to issue [PID][1]s specifically.

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
OpenID Connect session that the wallet started. (The `PidAttributeService` is
a part of the `pid_issuer` in the `wallet_server` crate, as opposed to a
separate HTTP server; we include it here as separate actor to clearly visualize
separate responsibilities.)

The protocol works as follows:

  * The wallet starts an OpenID Connect session at the `AuthServer` by sending
    it an Authorization Request, receiving an authorization code from the
    `AuthServer` in response;

  * Using the received authorization code, the wallet starts OpenID4VCI issuance
    in a so-called pre-authorized code flow by sending a `POST` request with the
    previously obtained code as a pre-authorized code in a Token Request to the
    `pid_issuer`;

  * The `pid_issuer` feeds the Token Request with the pre-authorized code to its
    `PidAttributeService` component. The `PidAttributeService` `POST`'s the
    Token Request to the `AuthServer`, transforming only the pre-authorized code
    in it to a normal authorization code but keeping the other parameters (such
    as the `state` and the PKCE `code_verifier`) in the Token Request as-is,
    thereby continuing the OpenID Connect session that the wallet previously
    started, and obtaining an `access_token`;

  * Using the resulting `access_token`, the `PidAttributeService` invokes the
    `/userinfo` endpoint of the `AuthServer` to retrieve the BSN, with which it
    does a query to the [BRP][7], resulting in the attributes to be issued;

  * The `pid_issuer` then generates the `c_nonce` and an `access_token` of its
    own, and a preview of the attestations as a custom addition to the
    OpenID4VCI protocol, all of which it returns to the wallet;

  * With the `access_token` and a valid set of proofs of possession (signatures
    over the `c_nonce` validating against the public keys that the wallet wants
    to have in its PID), the wallet can access the `batch_credential` endpoint
    of the `pid_issuer` to obtain the attestations.

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
        Wallet ->> WP: request PoPs with nonce<br/>(PerformIssuanceWithWua instruction)
        WP ->> Wallet: Return WUA and Signed PoP and PoA
        Wallet->>+WalletServer: POST /batch_credential(access_token, PoPs)
        note over Wallet: WUA and PoA are included here
        WalletServer->>WalletServer: verify proofs,  WUA and PoA
        WalletServer->>-Wallet: attestations
    deactivate Wallet
```

## References

Below you'll find a collection of links which we reference to through the
entire text. Note that they don't display when rendered within a website, you
need to read the text in a regular text editor or pager to see them.

[1]: https://eudi.dev/latest/annexes/annex-3/annex-3.01-pid-rulebook
[2]: https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-ID1.html
[3]: https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-ID1.html#name-pre-authorized-code-flow
[4]: https://openid.net/developers/how-connect-works
[5]: https://www.logius.nl/onze-dienstverlening/toegang/digid
[6]: https://github.com/minvws/nl-rdo-max
[7]: https://www.rvig.nl/basisregistratie-personen
[8]: disclosure-based-issuance
