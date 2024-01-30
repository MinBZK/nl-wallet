# Issuance with OpenID4VCI

## PID issuance

This diagram shows how we use OpenID4VCI in the pre-authorized code flow to issue the PID.

In this protocol, the wallet receives a (pre-authorized) code from the AuthServer (DigiD/rdo-max), which then has to be exchanged for the attributes to be issued at the Wallet Server. To deal with this, we introduce an actor in the diagram called the AttributeService, whose responsibility it is to produce the attributes to be issued given the pre-authorized code. In the case of PID issuance it can do this by talking OpenID with the AuthServer. (This actor is a library part of the WalletServer, as opposed to a separate HTTP server; we include it as separate actor here to separate responsibilities.)

The protocol works as follows:

- The wallet POSTs the code that it receives after DigiD not to the AuthServer, but as a pre-authorized code to the WalletServer.
- The WalletServer feeds the pre-authorized token to the AttributeService, which forwards the token to the AuthServer by performing a normal OpenID token POST request to it. Using the resulting `access_token` it invokes the `userinfo` endpoint to retrieve the BSN, with which it can do a BRP query, resulting in the attributes to be issued that are returned to the WalletServer. The WalletServer then generates the `c_nonce` and an `access_token` of its own, and returns these to the wallet.
- Along with the `access_token` and `c_nonce` we also return a preview of the attestations, as a custom addition to the OpenID4VCI protocol.
- When the wallet accesses the `batch_credential` endpoint with the `access_token` and a valid set of proofs of possession (signatures over the `c_nonce` validating against the public keys that the wallet wants to have in its PID), the WalletServer generates the attestations and returns them.

```mermaid
sequenceDiagram
    autonumber

    actor User
    participant OS
    participant Wallet
    participant WalletServer
    participant PidAttributeService
    participant AuthServer

    User->>+Wallet: click "issue PID"
    Wallet->>-OS: navigate to AuthServer/authorize?redirect_uri=...
    OS->>+AuthServer: GET /authorize?redirect_uri=...
    note over User, AuthServer: authenticate user with DigiD app
    AuthServer->>AuthServer: generate & store code
    AuthServer->>-OS: navigate /credential_offer(pre-authorized_code)
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
        Wallet->>Wallet: create PoPs by signing nonce using Wallet Provider
    Wallet->>+WalletServer: POST /batch_credential(access_token, PoPs)
    WalletServer->>-Wallet: attestations
    deactivate Wallet
```

## Generic issuance

For generic issuance, we can implement the AttributeService as follows:
  * The issuer feeds it a bunch of to-be-issued attestations (e.g. `Vec<UnsignedMdoc>`) and receives a fresh pre-authorized token in return, which it sends to the wallet using a UL or QR;
  * When the WalletServer calls `getAttributes(pre-authorized_code)` on the AttributeService, it looks up the attributes to be issued using the pre-authorized code and returns them.

This would look like the following diagram.

```mermaid
sequenceDiagram
    autonumber

    actor User
    participant OS
    participant Wallet
    participant WalletServer
    participant GenericAttributeService
    participant Issuer

    User->>+Issuer: start issuance flow
    Issuer->>Issuer: determine attributes to be issued
    Issuer->>+WalletServer: createSession(attributes)
    WalletServer->>+GenericAttributeService: createSession(attributes)
    GenericAttributeService->>GenericAttributeService: generate pre-authorized_code<br/>store attributes
    GenericAttributeService->>-WalletServer: pre-authorized_code
    WalletServer->>-Issuer: pre-authorized_code
    Issuer->>-OS: navigate /credential_offer(pre-authorized_code)
    OS->>Wallet: openWallet(code)
    activate Wallet
        Wallet->>+WalletServer: POST /token(pre-authorized_code)
        WalletServer->>+GenericAttributeService: getAttributes(pre-authorized_code)
        GenericAttributeService->>GenericAttributeService: lookup attributes using pre-authorized code
        GenericAttributeService->>-WalletServer: attributes
        WalletServer->>WalletServer: generate c_nonce, access_token
        WalletServer->>-Wallet: access_token, c_nonce, attestation_previews
        Wallet->>+User: Show attributes, ask consent
    deactivate Wallet
    User->>-Wallet: approve with PIN
    activate Wallet
        Wallet->>Wallet: create PoPs by signing nonce using Wallet Provider
    Wallet->>+WalletServer: POST /batch_credential(access_token, PoPs)
    WalletServer->>-Wallet: attestations
    deactivate Wallet
```
