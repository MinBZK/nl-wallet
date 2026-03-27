# Disclosure with OpenID4VP

This diagram shows how OpenID4VP is implemented in the NL Wallet Solution.
This implementation is based on [OpenID4VP draft 20][1], with
Proof of Association (PoA) as a custom addition.

## How disclosure is implemented

This diagram shows how OpenID4VP is implemented within the NL Wallet Solution.

Client authentication is done using the `x509_san_dns` Client Identifier Scheme.
Other Client Identifier Schemes are currently not supported.

The reponse mode is `direct_post.jwt`. Other response modes are currently not
supported. In more detail, the protocol works as follows:

### Disclosure using OpenID4VP
```{mermaid}
sequenceDiagram
    autonumber

    actor User
    participant Wallet
    participant RP as Verifier
    participant WalletServer as Verifier Service ('OV')

    User->>RP: start action to disclose attributes
    RP ->>+ WalletServer: initiate transaction
    WalletServer -->>- RP: session_token
    RP ->> Wallet: Authorization Request (request_uri) (opens wallet through Universal Link, or scan QR on another device)
    activate Wallet
        Wallet ->>+ WalletServer: retrieve Request Object (using request_uri)
        note over WalletServer, Wallet: Request Object (signed) contains a.o. 'client_metadata',<Br/> including JWKs with an ephemeral public key for encrypting the authorization response
        WalletServer -->>- Wallet: response with (signed) Request Object
        Wallet ->>+ User: request user consent for disclosure
        User ->>- Wallet: provide consent for disclosure of attributes (using PIN)

        Wallet ->> Wallet: Sign PoPs (using Wallet Backend)

        Wallet->>+ WalletServer: POST Authorization Response (VP Token) with presentations (encrypted to ephemeral public key)
        WalletServer ->> WalletServer: Decrypt VP Token<br>Verify contained attestations<br>Check that all requested attributes are received
        WalletServer -->>- Wallet: Response Object (contains 'redirect_uri')
    deactivate Wallet
    Wallet ->>+ User: redirect user to 'redirect_uri'
    User ->>- RP: proceed in verifier app/website
    RP ->>+ WalletServer: fetch response data (session_token)
    WalletServer -->>- RP: response data (verified attributes)
```

1. From the Verifier's website or app, the user decides to start an action that requires disclosed attributes.
2. The Verifier sends the attributes to be disclosed to the OV to initiate a disclosure session.
3. The OV returns a `session_token` created for this session.
4. The OpenID4VP Authorization Request `request_uri` parameter is put in a Universal Link or QR code, which is either opened or scanned by the Wallet.
5. The Wallet requests the Request Object from the given `request_uri`. The Request Object contains a.o. the `response_uri`, to which the wallet is to post the VP token later on, and an ephemeral public key to which the wallet can encrypt the VP token.
6. The OV returns the (signed) Request Object.
7. The Wallet presents the requested attributes and the identity of the verifier to the user for consent.
8. The User provides consent (using their PIN).
9. The Wallet collects the attributes to be sent and asks the Wallet Backend to sign the PoPs of the public keys of the attestations.
10. The VP Token is posted back to the `response_uri` of the OV.
11. The OV decrypts the received VP Token, verifies the contained attestations, and checks that it contains all requested attributes.
12. The OV returns a `redirect_uri` back into the verfier's website or app to the Wallet.
13. The Wallet redirects user to the Verifier using `redirect_uri`.
14. The User proceeds in the Verifier application.
15. The Verifier's website or app requests session results from the OV using `session_token`.
16. The OV returns session results with the verified attributes.

#### Key usage during dicslosure

During disclosure, the Wallet will interact with the Wallet Backend to sign the Proofs of Possessions (PoPs) that will be sent to the Verifier (Step 7 in the previous diagram). The sequence diagram below describes this process in detail. 

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

<!-- References -->

[1]: https://openid.net/specs/openid-4-verifiable-presentations-1_0-20.html