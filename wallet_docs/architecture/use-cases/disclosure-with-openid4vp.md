# Disclosure with OpenID4VP

## Generic disclosure
This diagram shows how OpenID4VP is implemented within the NL Wallet Solution.

[OpenID for Verifiable Presentations - draft 20](https://openid.net/specs/openid-4-verifiable-presentations-1_0-20.html) is used as basis for the implementation.

Client authentication is done using the `x509_san_dns` Client Identifier Scheme. Other Client Identifier Schemes are currently not supported.

The reponse mode is `direct_post.jwt`. Other response modes are currently not supported.

In more detail, the protocol works as follows:



### Disclosure using OpenID4VP
```{mermaid}
sequenceDiagram
    autonumber

    actor User
    participant RP as Verifier
    participant Wallet
     
    User->>RP: Start action to disclose attributes
    RP ->>+ Wallet: authorization request (request_uri) (open wallet through universal link or scan QR on another device)
    Wallet ->> RP: request Request Object (using request_uri)
    note over RP, Wallet: Request Object (signed) also contains 'client_metadata',<Br/> including jwks with an ephemeral key for encrypting the authorization response
    RP -->> Wallet: response with (signed) Request Object
    Wallet -->> User: request user consent for disclosure
    User ->> Wallet: provide consent for disclosure of attributes (using PIN)
  

    Wallet ->> Wallet: Sign response (using Wallet Backend)
  
    Wallet->>- RP: POST authorization response (VP Token) with presentations (JWE using ephemeral key)
    RP ->> RP: Validate VP Token
    RP ->>+ Wallet: response object containing [redirect_uri]
    Wallet ->>+ User: redirect user to [redirect_uri]
    User ->>+ RP: Proceed in verifier app
```

1. From Verifier's website or app, user decides to start an action that requires disclosed attributes
2. The OpenID4VP Authorization Request `request_uri` parameter is put in a Universal Link or QR code, which is either opened or scanned by the Wallet.
3. Wallet requests the request object from the given `request_uri`. The request object contains the `response_uri` 
4. Verifier returns request object (signed)  
5. The requested attributes and the identity of the verifier are presented to the user for consent
6. User provides consent (using PIN)
7. Wallet App collects the attributes to be sent and asks the Wallet Backend to sign the response
8. Response is posted back to Verifier (endpoint address was supplied in `response_uri` in the Request Object)
9. Verifier verifies the received VP Token
10. Verifier endpoint returns a `redirect_uri` back into the verfier's website or app to the Wallet
11. Wallet redirects user to Verifier using `redirect_uri`.
12. User proceeds in Verifier application using the data from the disclosure session

#### Key usage during dicslosure

During disclosure, the Wallet will interact with the backend to sign the response that will be sent to the Verifier. (Step 7 in the previous diagram) The sequence diagram below describes this process in detail. 

```{mermaid}
sequenceDiagram
    autonumber

    participant Wallet as WalletApp (core)
    participant WB as Wallet Backend 
    participant HSM as HSM
    participant DB as WB Database

    Wallet ->>+ WB: instruction: Sign(message[], key_identifiers, [PoA nonce])
    WB ->>+ DB: retrieve key(s) by key_identifiers
    DB -->>- WB: wrapped_key(s) from account (from key_identifiers)
    WB ->>+ HSM: sign message(s) using wrapped_key(s)
    HSM ->> HSM: unwrap wrapped_key(s) using attestationWrappingKey and<br/> use unwrapped key (attestationPrivateKey) for signing message(s)
    HSM ->>- WB: signed message
    opt PoA requested (in case of multiple messages)
        WB ->>+ HSM: sign PoAs for keys, using PoA nonce  
        HSM -->>- WB: signed PoAs
    end
    WB -->>- Wallet: instruction response: signed messsage(s), PoA  

```



### Disclosure using OpenID4VP (using OV software component)

NL-Wallet provides Relying Parties with a software component that will unburden the RP application from implmenting the OpenID4VP Protocol. The sequence diagram below describes the steps in the disclosure flow when the OV software component is integrated at the Relying Party. 

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
    Wallet ->> User: redirect to the redirect URI (session_token)
    User ->> RP: proceed in Verifier App
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
13. User proceeds in verifier app
14. Verifier's website or app requests session results from 'OV' using `session_token`
15. 'OV' returns session results