# Disclosure with ISO 18013-5 (close proximity)

This diagram shows how close proximity disclosure is implemented. The flow assumes QR-based device engagement followed by BLE-based device retrieval. This combination is consistent with ISO/IEC 18013-5 proximity presentation, where QR code is one of the standard engagement channels and BLE is one of the standard proximity retrieval transports. For the holder, this combination forms the minimal supported method set for compliance with the selected close proximity disclosure profile.


## High level overview

```{mermaid}
sequenceDiagram
    %% Force ordering by explicitly setting up participants
    actor user
    participant wallet
    participant reader

    title mdoc disclosure using ISO18013-5:2021 (high level overview)

    user->>+wallet: Presses "start action: show QR code"

    wallet->>wallet: Generate eDeviceKey, DeviceEngagement start BLE server, render QR code

    wallet-->>user: Render "show this QR to reader"

    activate reader
    Note over user, reader: Reader scans the QR with the DeviceEngagement

    reader-->>wallet: Connect to BLE
    wallet-->>user: Update "connected"

    reader->>wallet: SessionEstablishment { eReaderKey, encrypted DeviceRequest }

    wallet->>wallet: Compute SessionTranscript, session key, decrypt DeviceRequest

    wallet-->>user: Update "deviceRequestReceived"

    user-->>wallet: Frontend triggers "Continue close proximity disclosure"

    Note right of wallet: The DeviceRequest contains the ReaderAuth

    wallet->>wallet: Verify ReaderAuth, find candidates from storage

    wallet-->>user: Render "reader information and attributes to be shared"

    user->>wallet: Approve with pin

    wallet->>wallet: Sign DeviceAuthenticationBytes using WSCA, construct DeviceResponse and PoA

    wallet->>reader: Encrypted DeviceResponse with PoA
    wallet-->>-user: Render "attributes disclosed"

    reader-->>reader: verify DeviceResponse
```
1. The user starts the proximity flow in the wallet by choosing the action that shows a QR code for disclosure. This is the entry into the ISO 18013-5 proximity presentation flow represented in the diagrams.

2. The wallet generates a fresh ephemeral device key (`eDeviceKey`), starts the BLE server, and encodes the resulting `DeviceEngagement` into a QR code. In ISO 18013-5, device engagement carries the information needed to set up secure close proximity retrieval and QR code is one of the engagement technologies defined for that phase.

3. The user presents that QR code to the reader. The reader scans it, extracts the `DeviceEngagement`, and uses the advertised connection data to connect to the wallet over BLE. The wallet then updates the UI to show that the device is connected.

4. Once the transport layer is available, the reader sends a `SessionEstablishment` message that contains its ephemeral public key (`eReaderKey`) and an encrypted `DeviceRequest`. On the reader side, this is the point where the reader has already prepared its request and derived the session keys needed to protect the message.

5. After receiving `SessionEstablishment`, the wallet computes the `SessionTranscript`, derives the same session keys, and decrypts the `DeviceRequest`. The session transcript is a central input for both reader authentication and later device authentication.

6. When the request has been decrypted successfully, the wallet notifies the UI that the device request has been received. The frontend then triggers continuation of the close-proximity disclosure flow, at which point the wallet has both the request payload and the transcript material needed to evaluate whether the request is authentic and can be fulfilled.

7. The wallet verifies the `ReaderAuth` contained in the `DeviceRequest` and determines which locally stored attributes could fulfill the request and also if the reader is allowed to request these attributes. In ISO 18013-5 terms, `ReaderAuth` binds the request to the current `SessionTranscript` and the request bytes, so the wallet can check that the request really came from the reader identity associated with the supplied certificate chain.

8. The wallet shows the reader information and the attributes that would be disclosed, so that the user can see what is being shared and also the identity information of the party which will receive the data.

9. The user reviews the request and approves disclosure with a PIN. The approval step is the user-authorisation gate before the wallet is allowed to release the requested attributes.

10. The wallet signs the `DeviceAuthenticationBytes` using the `WSCA`, constructs the `DeviceResponse`, and prepares a `PoA` where the requested presentation requires proof that multiple attestations are associated with the same secure cryptographic environment. `DeviceAuthenticationBytes` cryptographically binds the disclosed device-signed data to the `SessionTranscript` and document type, which is what the reader will later verify.

11. The wallet sends the encrypted `DeviceResponse` with `PoA` back to the reader and updates the UI to show that the attributes have been disclosed. The reader then verifies the response, including the authenticated device-signed data and any accompanying proof material.


## Detailed sequence diagram

The next diagram expands the same disclosure into the internal responsibilities of the app UI layer, the shared wallet core, the native platform support layer, and the reader application. It also shows the main cancellation branches that can occur before disclosure is completed.

```{mermaid}
sequenceDiagram
    %% Force ordering by explicitly setting up participants
    actor user
    box rgb(211,211,211) NL Wallet app
        participant wallet_app
        participant wallet_core
        participant platform_support
    end
    participant reader_app

    title mdoc disclosure using ISO18013-5:2021

    user->>+wallet_app: Presses "Start action: Show QR code"

    wallet_app->>+wallet_core: startProximityDisclosure
    wallet_core->>+platform_support: startQrEngagement
    platform_support->>platform_support: Generate eDeviceKey, start BLE server
    platform_support->>wallet_core: DeviceEngagement
    wallet_core->>wallet_app: QrContents
    wallet_app->>wallet_app: Render QR code

    wallet_app-->>user: Render "Show this QR to reader"

    activate reader_app
    Note over user, reader_app: Reader scans the QR with the DeviceEngagement

    reader_app-->>platform_support: Connect to BLE server
    platform_support-->>wallet_core: Update: connected
    wallet_core-->>wallet_app: Update: connected
    wallet_app-->>user: Render "Receiving request"

    reader_app->>platform_support: SessionEstablishment { eReaderKey, encrypted DeviceRequest }

    platform_support->>platform_support: Compute session key, decrypt DeviceRequest
    platform_support-->>wallet_core: Update: SessionEstablished { DeviceRequest, SessionTranscript }
    Note over wallet_core, reader_app: The DeviceRequest contains the ReaderAuth

    wallet_core-->>wallet_app: Update: DeviceRequestReceived

    wallet_app->>wallet_app: Navigate to DisclosureScreen
    wallet_app-->>user: Render "Fetching disclosure request"

    wallet_app-->>wallet_core: continueCloseProximity()

    wallet_core->>wallet_core: Verify ReaderAuth
    wallet_core->>wallet_core: Find candidates from storage

    alt Any of the requested attributes are unavailable
        wallet_core->>wallet_core: logCancelled(DeviceRequest)

        wallet_core->>wallet_app: Attributes not found, ItemsRequest
        wallet_app-->>user: Render Requested attributes not available"
        user->>wallet_app: Ok
        wallet_app->>wallet_core: cancelDisclosure()
        wallet_core->>platform_support: Send session termination
        platform_support->>reader_app: StatusMessage { status: 20 }

        platform_support->>platform_support: Stop BLE
    end

    Note over wallet_core, wallet_app: The request is enriched with the<br>actual attribute values from storage

    wallet_core-->>wallet_app: StartDisclosureResult & candidates

    wallet_app-->>user: Render "reader information"

    alt Disapprove the relying party
        user->>wallet_app: Disapprove
        wallet_app-->>user: Render "disclosure aborted"
        wallet_app->>wallet_core: cancelDisclosure()
        wallet_core->>wallet_core: logCancelled(DeviceRequest)
        wallet_core->>platform_support: Send session termination
        platform_support->>reader_app: StatusMessage { status: 20 }

        platform_support->>platform_support: Stop BLE
    end

    user->>wallet_app: Approve

    wallet_app-->>user: Render "attributes to be shared"

    alt Disapprove sharing the requested attributes
        Note over user, reader_app: Same as in [disapprove the relying party]
    end

    user->>wallet_app: Approve

    wallet_app-->>user: Render "Enter PIN"
    user->>wallet_app: PIN

    wallet_app->>wallet_core: acceptDisclosure(pin, selected)

    wallet_core->>wallet_core: Sign DeviceAuthenticationBytes and PoA using WSCA

    wallet_core->>wallet_core: Construct DeviceResponse
    wallet_core->>platform_support: sendDeviceResponse(DeviceResponse)
    platform_support->>reader_app: Encrypted DeviceResponse

    platform_support->>platform_support: Stop BLE

    reader_app-->>reader_app: Verify DeviceResponse

    wallet_core->>wallet_core: logSuccess(Session)
    wallet_core-->>-wallet_app: Success

wallet_app-->>-user: Render "attributes disclosed"
```


1. The user starts the flow in `wallet_app`, which immediately delegates the protocol work to `wallet_core` through `startProximityDisclosure()`.

2. `wallet_core` asks `platform_support` to `startQrEngagement`. The platform layer generates a fresh `eDeviceKey` and starts the BLE server, then returns the resulting `DeviceEngagement` to `wallet_core`. `wallet_core` forwards it as `QrContents` to `wallet_app`, which renders the QR code and prompts the user to show it to the reader.

3. The reader scans the QR code, extracts the `DeviceEngagement`, and uses the advertised connection data to open a BLE connection to `platform_support`. The transport state then flows upward from `platform_support` to `wallet_core` and on to `wallet_app`, so the UI can reflect the transport setup before session establishment begins.

4. The reader then sends `SessionEstablishment { eReaderKey, encrypted DeviceRequest }` to `platform_support`. The native layer derives the session key, decrypts the `DeviceRequest`, and returns `SessionEstablished { DeviceRequest, SessionTranscript }` to `wallet_core`. In protocol terms, this is the transition from transport setup into protected request processing.

5. `wallet_core` signals `DeviceRequestReceived` to the UI, after which the UI calls `continueCloseProximity()`. The core then verifies `ReaderAuth` and looks up candidate attributes in storage that could fullfill the request. `ReaderAuth` is the reader-side authentication proof that binds the request to the current session.

6. If any requested attributes are unavailable, the core stops before user consent. It logs the cancellation, tells the UI which requested items cannot be satisfied, and, after user acknowledgement, sends a session-termination message and stops BLE. No user approval is requested for a request that cannot be fulfilled.

7. If the request can be satisfied, `wallet_core` enriches the request with the actual local attribute values and returns `StartDisclosureResult` plus the candidate set to the UI. The purpose of this step is to move from a purely syntactic request to a user-facing disclosure choice based on real wallet contents.

8. `wallet_app` first renders reader information. This is the relying-party review step, where the user can decide whether the requesting reader is acceptable before looking at the attribute disclosure.

9. If the user rejects the reader, the flow is cancelled. The core logs the cancellation, sends a session-termination message to the reader side and stops the BLE server. This branch prevents any further disclosure processing after requester rejection.

10. If the user accepts the reader, the app renders the attributes to be shared. The user then gets a second approval point focused specifically on the requested claims. This corresponds to selective disclosure and explicit user authorisation before release.

11. If the user rejects attribute sharing, the implementation follows the same cancellation path as reader rejection: cancel, terminate the session, and stop BLE.

12. If the user accepts the disclosure of the requested attributes, the user can also select which attestation to share the attributes from (in the case that there are multiple). After this, the app asks for the PIN and calls `acceptDisclosure(pin, selected)`. At that point, `wallet_core` performs the critical cryptographic step: it signs `DeviceAuthenticationBytes` and, where applicable, `PoA` using the `WSCA`, and then constructs the `DeviceResponse`.

13. `wallet_core` hands the completed response to `platform_support`, which sends the encrypted `DeviceResponse` to the reader application and then stops the BLE server. The reader verifies the response, `wallet_core` logs the successful session, and `wallet_app` renders the final "attributes disclosed" screen.

## Key usage during disclosure

Key usage is the same as in
[disclosure with OpenID4VP](disclosure-with-openid4vp).
