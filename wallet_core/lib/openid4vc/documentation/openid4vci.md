# Introduction to OpenID4VCI

OpenID4VCI is a protocol for issuance of attestations. The version of the
specification implemented here may be found
[here](https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html).

It aims to be generic over different attestation formats, such as the mdoc
attestation format and
[SD-JWT](https://datatracker.ietf.org/doc/draft-ietf-oauth-selective-disclosure-jwt/).
The standard therefore only defines protocols (although it includes a number of
examples using the mdoc, SD-JWT and JSON-LD VC formats). By contrast, the
[ISO mdoc](../../mdoc/documentation/mdoc.md) specifications (ISO 18013-5,
23220-3, 23220-4) define both an attestation format (COSE over `IssuerSigned`)
and protocols with which to issue and verify them.

A note on terminology: In the OpenID4VCI/OpenID4VP specs, a "Verifiable
Credential" and just "credential" is what we call an attestation. Here
"Verifiable Credential" must be taken in a broad sense and not to exclusively
mean [W3C Verifiable Credentials](https://www.w3.org/TR/vc-data-model/): if it
contains a (reference to a) user public key and user data signed by a trusted
issuer, then it may be considered a Verifiable Credential.

## Relation to OAuth and OpenID Connect

A short introduction to OAuth/OpenID Connect is given [here](./openid.md).

OpenID4VCI is in terms of protocol messages a superset of OAuth. However, the
roles played by the actors in the protocol as well as what protocol message gets
sent to whom is slightly different than in OAuth.

By contrast, OpenID Connect is a full superset of OAuth, both in terms of
protocol messages and who sends which messages to whom. OpenID4VCI is not
modeled as a superset of OpenID, as it issues a Verifiable Credential to the end
user instead of an ID token to (the webserver backend of) a Client.
Consequentially, in OAuth and OpenID the User Agent (the browser) receives the
Authorization Code while the Access Token is sent only to the Client. In
OpenID4VCI, instead the Wallet receives both the Authorization Code and the
Access Token.

## Protocol flow

The OpenID4VCI protocol has the following phases.

1. The issuer transmit a Credential Offer to the Wallet, e.g. in the form of a
   QR code, which contains Credential Configuration Identifiers for a set of
   offered credentials. This Credential Offer also determines which flow to use
   in step 3.
2. Based on the issuer URL in this Credential Offer, the Wallet retrieves both
   the the Issuer Metadata and Oauth 2.0 Authorization Server Metadata from the
   issuer, in order to determine the relevant HTTP endpoint paths and to obtain
   details about of the offered credentials.
3. The Wallet obtains an Authorization Code in one of the following two ways:
    1. Using the Authorization Code Flow defined in OpenID4VCI, by sending an
       Authorization Request to the Authorization Endpoint and receiving an
       Authorization Response as in OAuth 2.0.
    2. Using the Pre-Authorized Code Flow, i.e. in some issuer-specific way not
       covered by OpenID4VCI (in this case the Authorization Code is called the
       "Pre-Authorized Code" but it performs the same role).
4. The Wallet exchanges the Authorization Code for an Access Token at the Token
   Endpoint using an OAuth Token Request, receiving an OAuth Token Response
   containing the Access Token.
5. If the Issuer Metadata contained a Nonce Endpoint, the Wallet calls this
   endpoint for each credentials it wishes to have issued. The Wallet will have
   to sign this nonce with its attestation private keys (that is, the private
   keys of which it wants the corresponding public keys to be put in the issued
   credentials).
6. After creating PoPs (Proofs of Possessions) in the form of JWTs, which may
   include the nonce retrieved in the previous step, the Wallet sends these to
   an OpenID4VCI-specific Credential Endpoint. It calls the Credential Endpoint
   once per credential it wishes to have issued, with each invocation resulting
   in one or more copies of the same credential data, determined by the amount
   of proofs the Wallet sends. This endpoint is an OAuth 2.0 Protected Resource,
   i.e., requires the Access Token in the `Authorization` header. The issuer
   verifies the PoP JWTs and responds with the issued credential copies.

A sequence diagram of the pre-authorized code flow looks as follows. In this
flow, the `code` is renamed to `pre-authorized_code` (but otherwise it functions
in the same way). Note that the HTTP paths in this diagram are just examples.

```mermaid
sequenceDiagram
    autonumber

    actor User
    participant OS
    participant Wallet
    participant Issuer

    note over User, Issuer: authenticate user (out of scope of OpenID4VCI pre-authorized code flow)
    Issuer->>OS: openWallet(credential_offer)
    OS->>Wallet: openWallet(credential_offer)
    activate Wallet
        Wallet->>+Issuer: GET /issuer_metadata
        Issuer->>-Wallet: OpenID4VCI Issuer Metadata

        Wallet->>+Issuer: GET /oauth_metadata
        Issuer->>-Wallet: OAuth 2.0 Authorization Server Metadata

        Wallet->>+Issuer: POST /token (pre-authorized_code)
        Issuer->>Issuer: lookup code
        Issuer->>-Wallet: access_token

        loop Every credential
            Wallet-->>+Issuer: GET /nonce
            Issuer->>Issuer: generate nonce
            Issuer-->>-Wallet: nonce

            Wallet->>+Issuer: POST /credential (access_token, PoPs)
            Issuer->>Issuer: sign credential
            Issuer->>-Wallet: attestation copies
        end
    deactivate Wallet
```

Sequence diagrams showing full details of the implementation in this crate can
be found
[here](../../../../wallet_docs/architecture/use-cases/issuance-with-openid4vci.md).

## Comparison with the mdoc issuance protocol

### Similarities

- At its core, the protocol works the same as any attestation issuance protocol
  necessarily has to work: for each attestation (copy) that is issued, the
  holder has to sign a random nonce generated by the issuer with the private key
  whose public key it wants to have in the attestation, thereby proving
  possession to the issuer of the private key.
- There are as many private keys involved as there are attestation (copies) to
  be issued, but per credential there is only a single nonce that the holder
  signs with all private keys.
- The protocol supports issuance of multiple attestation copies simultaneously,
  while issuance of multiple credentials requires subsequent calls to the
  Credential Endpoint.

### Differences

- An mdoc issuance session uses a session token that is included in the URL. The
  OpenID4VCI endpoints include no session token; instead the session is
  identified by either the authorization code or the access token, depending on
  which endpoint is invoked. These are sent as an HTTP header instead of in the
  URL.
- The mdoc issuance protocol always sends all data CBOR-encoded in the body of a
  HTTP POST request or response. In the OpenID4VCI protocol, the HTTP requests
  are sometimes POST and sometimes GET; data is sent sometimes in the POST body
  and sometimes in a HTTP header; and data is sometimes JSON-encoded and
  sometimes URL-encoded. All of this depends on the specific request being done
  and the data being sent.
- The protocol does not provide a message that informs the holder of the
  attestations and attribute values it is about to receive, before it actually
  receives them. Instead, the protocol assumes that the user is informed of this
  when the issuer has control over the flow when it is authenticating the user.

## Specifics of this implementation

- Since the OpenID4VCI protocol is structured as a superset of OAuth, this
  implementation is as well; OpenID4VCI-specific types of protocol messages
  contain the base OAuth 2.0 message while expanding upon it with extra fields.
  Additionally, the protocol messages in this implementation are such that they
  can be used in both OpenID4VCI servers and OpenID(4VCI)/OAuth clients.
- This implementation uses the DPoP (Demonstrating Proof of Possession,
  [RFC 9449](https://datatracker.ietf.org/doc/html/rfc9449)) mechanism, which
  defends against certain replay attacks by making the wallet use an ephemeral
  ECDSA private key in both calls to the issuer:
    - Just before the wallet sends the pre-authorized code to the Token
      Endpoint, it generates a new ECDSA public/private keypair, and then signs
      the public key using the private key;
    - When the wallet sends the pre-authorized code to the Token Endpoint, it
      includes the signed public key to make it known to the issuer;
    - When the holder requests the attestations by sending the access token and
      its signatures over the nonce to the Credential Endpoint, it also signs
      and includes the access token with the ephemeral ECDSA private key. The
      issuer verifies the signature using the public key from the Token Endpoint
      invocation.
- Normally in both OAuth and OpenID(4VCI), the user gives user consent when the
  User Agent has navigated to the Authorization Endpoint and the user has
  authenticated themselves. In OpenID4VCI, it is additionally assumed that at
  this moment in the flow, i.e. during the issuer-controlled part, the user is
  informed of the attribute names and maybe the values that it will receive.
  However, in this implementation instead we wish to ask for the user consent in
  the Wallet App itself, to unify the UX of this experience. Therefore, user
  consent is implemented here as follows:
    - The issuer-controlled part is not assumed to inform the user of the
      attribute values; instead it is only responsible for authenticating the
      user, as it was already.
    - This implementation adds a custom Credential Preview endpoint, the path to
      which is included in the Issuer Metadata.
    - After receiving the Access Token, the Wallet App retrieves the Credential
      Preview from the Issuer, which contains all claims of all the credentials
      offered in the issuance session.
    - The Wallet App shows the attestation previews to the user and asks for
      their consent. Only then does the Wallet app invoke the Credential
      Endpoint to obtain the attestations. (By contrast, normally in
      OAuth/OpenID(4VCI) the Authorization Code / Pre-Authorized Code itself
      represents the user consent, so that the OAuth 2.0 Potected Resource would
      be invoked immediately after receiving the Access Token.)
    - The user can choose to abort issuance, after which the Wallet App will not
      call the Credential Endpoint.
- As OpenID4VCI does not contain a provision to distribute SD-JWT VC Type
  Metadata, this implementation includes a Credential Metadata Endpoint. The
  Issuer anounces this endpoint in the Issuer Metadata and the Wallet App
  retrieves SD-JWT VC Type Metadata documents for all issued credentials.
- This implementation is currently not compatible with potential other
  implementations that are unaware of (and thus don not implement) both the
  Credential Preview and Credential Metadata endpoints; this is left for later.
- In the OAuth/OpenID(4VCI) protocols the Authorization and Token Requests that
  the client sends are not JSON-encoded but instead URL-encoded (as they are
  (sometimes) sent as the query parameter in the URL). In this implementation,
  we deal with those the same as we deal with other (JSON-encoded) protocol
  messages: they are implemented as a `struct` and are (de)serialized to/from
  with `serde` (contrary to other implementations with often string-manipulate
  the URL parameters), using `serde_urlencoded`.
    - NB: Consequentially, missing or incorrect parameters in protocol messages
      are detected early on by the deserializer and might not result in the
      appropriate error message.
