# Disclosure flow

The diagrams here show the interactions that take place when the user discloses information.

## 5.1 Disclosure

Work in progress. Initial interpretation of the disclosure flow.

```mermaid
sequenceDiagram
    %% Force ordering by explicitly setting up participants
    actor user
    participant platform
    participant wallet_app
    participant wallet_core
    participant relying_party
    participant wallet_provider
    title Disclosure [5.1]

    user->>wallet_app: scan discolsure qr
    activate user
        wallet_app->>+wallet_app: decodeQr(qr)
        wallet_app->>wallet_core: startDisclosure(uri)
        activate wallet_core
            wallet_core->>+relying_party: getDisclosureRequest(uri)
            opt server error
                relying_party-->>wallet_core: server error
                wallet_core->>wallet_app: error
                wallet_app->>user: show server error
            end
            relying_party-->>-wallet_core: disclosureRequest
            Note over wallet_core, relying_party: This request contains the requesting<br>party and requested attributes.
            alt requested attributes unavailable
                wallet_core->>wallet_app: requested attributes not found
                wallet_app->>user: render disclosure failed
            end
            wallet_core-->>wallet_app: disclosureRequest
            Note over wallet_core, wallet_app: The request is enriched<br>with the actual attribute values
            wallet_app->>user: render requesting party
            alt disapprove relying party
                user->>wallet_app: deny relying party
                wallet_app->>wallet_core: cancelDisclosure()
                wallet_core->>wallet_core: logCancelled(disclosureRequest)
                wallet_app->>user: render disclosure aborted
            end
            user->>wallet_app: approve relying party
            wallet_app->>user: render requested attributes and values
            alt disapprove sharing requested attributes
                user->>wallet_app: deny sharing
                wallet_app->>wallet_core: cancelDisclosure()
                wallet_core->>wallet_core: logCancelled(disclosureRequest)
                wallet_app->>user: render disclosure aborted
            end
            user->>wallet_app: approve sharing of attributes
            wallet_app->>user: render request pin
            user->>wallet_app: pin
            wallet_app->>wallet_core: approveDisclosureRequest(pin)
            wallet_core->>+wallet_provider: prepareAttributes(attrs) (??)
            opt server error
                Note over wallet_provider,user: same as server error above
            end
            wallet_provider-->>-wallet_core: signedAttributes (??)
            wallet_core->>+relying_party: disclose(signedAttributes)
            opt server error
                Note over relying_party,user: same as server error above
            end
            relying_party-->>-wallet_core: success
            wallet_core->>wallet_core: logSuccess(disclosureRequest)
            wallet_core-->>wallet_app: success
        deactivate wallet_core
        wallet_app->>-user: render attributes disclosed
    deactivate user
```
