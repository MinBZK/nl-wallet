# Disclosure with ISO 18013-5 (close proximity)

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

## Detailed sequence diagram

```{mermaid}
sequenceDiagram
    %% Force ordering by explicitly setting up participants
    actor user
    box rgb(64,64,64) NL Wallet app
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

    reader_app->>platform_support: SessionEstablishment { eReaderKey, encrypted DeviceRequest }

    platform_support->>platform_support: Compute session key, decrypt DeviceRequest
    platform_support-->>wallet_core: Update: SessionEstablished { DeviceRequest, SessionTranscript }
    Note over wallet_core, reader_app: The DeviceRequest contains the ReaderAuth

    wallet_core-->>wallet_app: Update: DeviceRequestReceived
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

## Key usage during disclosure

Key usage is the same as in
[disclosure with OpenID4VP](../disclosure-with-openid4vp).
