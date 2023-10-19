# Session flow

## Disclosure

The following diagram shows a disclosure session from the perspective of the RP, showing in particular all interactions between the different components that the RP must be running:

- The RP runs an `RP_webserver` containing its own business logic.
- The RP has an `RP_website` that the `user` interacts with using their browser. The `RP_webserver` serves as the backend for this website.
- The RP has also deployed an instance of the `RP_server` in its infrastructure, which when instructed to by the `RP_webserver` performs disclosure with the user's `wallet`.

This diagram does not distinguish between the wallet's GUI and its Rust core, and it does not take the Wallet Provider protocol into account. For that, see [here](./disclosure.md). Additionally, this protocol shows only the happy flow in which the user's wallet possesses the attributes requested by the RP, and the user consents to disclose them.

It uses the following mdoc-specific data structure names:
- `ReaderEngagement` (step (6)): contains (among others) the URL at which the wallet can perform the mdoc disclosure protocol with the `RP_server`.
- `DeviceEngagement` (step (15)): requests the following message (`DeviceRequest`) from the `RP_server`.
- `DeviceRequest` (step (16)): contains the attributes that the RP requests from the wallet, as well as the RP authentication (an X.509 certificate containing the RP's name and any other data about the RP that the GUI needs to be able to display (such as the RP's reason for the disclosure), combined with a proof of possession of that certificate).
- `DeviceResponse` (step (20)): contains the attributes disclosed by the wallet, including a cryptographic proof of possession of them so that the RP can verify their authenticity.

```mermaid
sequenceDiagram
    autonumber

    %% Force ordering by explicitly setting up participants
    actor user
    participant wallet
    participant RP_website
    participant RP_webserver
    participant RP_server

    user->>RP_website: click button [e.g. "login" or "prove older than 18"]
    RP_website->>RP_webserver: buttonClicked()
    activate RP_webserver
        RP_webserver->>RP_webserver: compute attributes to be disclosed
        RP_webserver->>RP_server: startDisclosure(requested_attributes)
        activate RP_server
            RP_server->>RP_server: generate session_token,<br>store session
            RP_server-->>RP_webserver: ReaderEngagement, session_token
        deactivate RP_server
        RP_webserver-->>RP_website: ReaderEngagement, session_token
    deactivate RP_webserver
    RP_website->>RP_website: encode ReaderEngagement into QR or Universal Link (UL)
    RP_website-xRP_server: startPolling(session_token) / openWebsocket(session_token)
    activate RP_server
        alt cross device
            user->>wallet: activate QR scanner
            wallet->>RP_website: scan QR
            RP_website->>wallet: QR
        else same device
            RP_website->>wallet: navigate to UL
        end
        wallet->>wallet: parse ReaderEngagement from QR/UL
        critical mdoc disclosure protocol
            wallet->>+RP_server: startSession(DeviceEngagement)
            RP_server-->>-wallet: DeviceRequest(RP_name, requested_attributes)
            wallet->>user: show RP + requested_attributes
            user->>wallet: consent
            wallet->>wallet: compute disclosure [with Wallet Provider]
            wallet->>+RP_server: DeviceResponse(mdocs, disclosed attributes, proof)
            RP_server->>RP_server: verify
            RP_server-->>-wallet: OK
        end
        wallet->>user: show success
        RP_server--xRP_website: [over poll/websocket] sessionFinished()
    deactivate RP_server
    alt same device
        user->>wallet: OK
        wallet->>RP_website: navigate to return URL
    end
    RP_website->>+RP_webserver: handleSessionFinished()
    activate RP_server
    RP_webserver->>+RP_server: getAttributes(session_token)
    RP_server-->>-RP_webserver: disclosed_attributes
    RP_webserver->>RP_webserver: handle disclosed_attributes
    RP_webserver-->>-RP_website: user is authenticated
```

N.B. How the wallet determines or obtains the URL to return to in step 26 is yet to be determined.

## Issuance

Attribute issuance is to a large extent very similar to disclosure. Calling the rightmost three agents `issuer_website`, `issuer_webserver` and `issuer_server` respectively, issuance differs from disclosure in the following ways. 

1. Instead of step (3) in the sequence diagram above, the `issuer_webserver` needs to somehow authenticate the user as being a valid recipient for the attributes to be issued, as well as determine the contents of those attributes. The process for this will differ for each issuer. For example, the PID issuer authenticates the user using DigiD and retrieves the attributes to be issued from the BRP. This step is thus part of the business logic of the `issuer_webserver` and the specifics are out of scope here.
1. The protocol messages exchanged are replaced by those of the issuance protocol.
1. At the end of a disclosure session, the RP's application flow generally proceeds to a next part in which it somehow uses the authenticated `disclosed_attributes`. By contrast, at the end of issuance the `issuer_webserver` receives only a status from the `issuer_server` indicating issuance was succesful, and the issuer's application flow generally stops.

The sequence diagram follows. Note that all arrows are the same as in the diagram above; the only differences are in the descriptions above the arrows.

See also [the PID issuance sequence diagrams](./wallet_personalisation.md), which does not distinguish the various issuer components but does takes into account the wallet's GUI and Rust core, as well as the Wallet Provider protocol.

```mermaid
sequenceDiagram
    autonumber

    %% Force ordering by explicitly setting up participants
    actor user
    participant wallet
    participant issuer_website
    participant issuer_webserver
    participant issuer_server

    user->>issuer_website: click button [e.g. "issue attributes"]
    issuer_website->>issuer_webserver: buttonClicked()
    activate issuer_webserver
        issuer_webserver->>issuer_webserver: authenticate user and<br>compute attributes to be issued
        issuer_webserver->>issuer_server: startIssuance(unsigned_attributes)
        activate issuer_server
            issuer_server->>issuer_server: generate session_token,<br>store session
            issuer_server-->>issuer_webserver: ServiceEngagement, session_token
        deactivate issuer_server
        issuer_webserver-->>issuer_website: ServiceEngagement, session_token
    deactivate issuer_webserver
    issuer_website->>issuer_website: encode ServiceEngagement into QR or Universal Link (UL)
    issuer_website-xissuer_server: startPolling(session_token) / openWebsocket(session_token)
    activate issuer_server
        alt cross device
            user->>wallet: activate QR scanner
            wallet->>issuer_website: scan QR
            issuer_website->>wallet: QR
        else same device
            issuer_website->>wallet: navigate to UL
        end
        wallet->>wallet: parse ServiceEngagement from QR/UL
        critical mdoc issuance protocol
            wallet->>+issuer_server: StartIssuingMessage()
            issuer_server-->>-wallet: RequestKeyGenerationMessage(issuer_name, unsigned_attributes, challenge)
            wallet->>user: show issuer_name + unsigned_attributes
            user->>wallet: consent
            wallet->>wallet: sign challenge [with Wallet Provider]
            wallet->>+issuer_server: KeyGenerationResponseMessage(public_keys, responses)
            issuer_server->>issuer_server: verify
            issuer_server-->>-wallet: mdocs
        end
        wallet->>user: show success
        issuer_server--xissuer_website: [over poll/websocket] sessionFinished()
    deactivate issuer_server
    alt same device
        user->>wallet: OK
        wallet->>issuer_website: navigate to return URL
    end
    issuer_website->>+issuer_webserver: handleSessionFinished()
    activate issuer_server
    issuer_webserver->>+issuer_server: getStatus()
    issuer_server-->>-issuer_webserver: OK
    issuer_webserver-->>-issuer_website: OK
```

## Combined disclosure and issuance

It is expected that for many future (Q)EAA issuers, the user authentication and attribute retrieval that happens in step
 (3) is done using the PID attributes, which the wallet therefore discloses to the issuer just before issuance. Essentially, in the issuance sequence diagram above step (3) is replaced by the entire disclosure sequence diagram.

In more detail, the following happens.

1. The disclosure flow happens as seen in the first diagram, up and until step (29).
1. In step (30) of the disclosure flow, the `issuer_webserver` start issuance as seen in the second diagram from step (3), using the disclosed attributes to determine the attributes to be issued.
1. In step (20) of the issuance flow, the wallet needs to sign the challenge not only with the private keys of the mdocs that are being issued, but also with the private keys of the mdoc copies out of which it previously disclosed attributes in the disclosure phase. This convinces the issuer that all private keys involved, both those of the existing mdocs and the ones that are being issued, belong to one and the same wallet instance. (This prevents a possible attack vector in which two colluding wallets disclose attributes out of one wallet, and then receive the new mdocs in the other.)
