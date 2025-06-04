# Disclosure with OpenID4VP

## Generic disclosure
This diagram shows how OpenID4VP is implemented within the NL Wallet Solution.

[OpenID for Verifiable Presentations - draft 20](https://openid.net/specs/openid-4-verifiable-presentations-1_0-20.html) is used as basis for the implementation.

Client authentication is done using the `x509_san_dns` Client Identifier Scheme. Other Client Identifier Schemes are currently not supported.

The reponse mode is `direct_post.jwt`. Other response modes are currently not supported.

In more detail, the protocol works as follows:

```{mermaid}
sequenceDiagram
    autonumber

    actor User
    participant RP as Verifier
    participant WalletServer as Verifier Endpoint ('OV')
    participant Wallet
    User->>RP: Start action to disclose attributes
    RP->>WalletServer: initiate transaction
    WalletServer ->>RP: return session_token    
    RP ->>+ Wallet: authorization request (request_uri) (open wallet through universal link or scan QR on another device)
    Wallet ->> WalletServer: request Request Object (using request_uri)
    note over WalletServer: Request Object (signed) also contains 'client_metadata',<Br/> including jwks with an ephemeral key for encrypting the authorization response
    WalletServer ->> Wallet: response with (signed) Request Object
    Wallet ->>+ User: request user consent for disclosure
    User ->> Wallet: provide consent for disclosure of attributes (using PIN)
    Wallet->> WalletServer: authorization response (VP Token) (JWE using ephemeral key)
    WalletServer ->>WalletServer: Validate VP Token
    WalletServer ->> Wallet: authorization response (redirect_uri with session_token)
    Wallet ->> RP: redirect to the redirect URI (session_token)
    RP ->> WalletServer: fetch response data (session_token)
    WalletServer ->> RP: response data (verified attributes)

```


1. From Verifier's website or app, user decides to start an action that requires disclosed attributes
2. A request goes from Verifier App to OV to start a disclosure session
3. OV returns `session_token` created for this session
4. The OpenID4VP Authorization Request `request_uri` parameter is put in a UL or QR code, which is either opened or scanned by the Wallet.
5. Wallet requests the request object from the given `request_uri`. The request object contains the `response_uri` 
6. Verifier returns request object (signed)  
7. The requested attributes and the identity of the verifier are presented to the user for consent
8. User provides consent (using PIN)
9. Response is posted back to Verifier Endpoint ('OV') (endpoint address is supplied in `response_uri` within the Request Object)
10. 'OV' verifies the received VP Token
11. Verifier endpoint returns a `redirect_uri` back into the verfier's website or app
12. Redirect to verifier using `redirect_uri`.
13. Verifier's website or app requests session results from 'OV' using `session_token`
14. 'OV' returns session results
