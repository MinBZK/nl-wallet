# Pin validation

 Pin validation is comprised of two steps, first of the [local](#local-pin-validation) pin validation (when configuring the wallet) to prevent users from selecting a trivial pin. Secondly the [remote](#remote-pin-validation) pin validation, where the user provided pin is compared with the previously registerd pin by the `wallet_provider`.

## Local pin validation

This diagram illustrates the local validation that happens when the user configures a new pin, to make sure the user does not select a trivial pin like `000000`.

```{mermaid}
sequenceDiagram
    %% Force ordering by explicitly setting up participants
    actor user
    participant platform
    participant wallet_app
    participant wallet_core
    participant wallet_provider
    title Local Pin Validation [3.1]

    user->>wallet_app: enter new pin
    activate user
        wallet_app->>wallet_core: validate pin
        alt pin invalid
            wallet_core-->>wallet_app: pin error
            Note over wallet_core,wallet_app: TooFewUniqueDigits,<br/>SequentialDigits, Other
            wallet_app->>user: render error & request new pin
        else pin valid
            wallet_core-->>wallet_app: pin success
            wallet_app->>user: render success
        end
    deactivate user
```

## Remote pin validation

This diagram illustrates the remote validation, this occurs when the user has already configured her wallet and is e.g. trying to log in to the app.

```{mermaid}
sequenceDiagram
    %% Force ordering by explicitly setting up participants
    actor user
    participant platform
    participant wallet_app
    participant wallet_core
    participant wallet_provider
    title Remote Pin Validation [3.2]

    user->>wallet_app: enter existing pin
    activate user
        wallet_app->>wallet_core: unlock(pin)
        wallet_core->>wallet_core: getRegistration()
        alt no registration
            wallet_core-->>wallet_app: NotRegistered
            wallet_app->>user: show not registered
        end
        par sign challenge request
            wallet_core->>wallet_core: createChallenge(incrementAndStore(sequenceNumber))
            wallet_core->>platform: sign(challengeRequest)
            platform->>platform: signWithHwKey(challengeRequest)
            platform-->>wallet_core: signedChallengeRequest
        end
        wallet_core->>wallet_provider: requestChallenge(signedChallengeRequest)
        opt server error
        wallet_provider-->>wallet_core: server error
            wallet_core-->>wallet_app: error
            wallet_app->>user: show server error
        end
        wallet_provider-->>wallet_core: challenge
        par sign instruction
            wallet_core->>wallet_core: signWithPinKey(pin, challenge, incrementAndStore(sequenceNumber))
            wallet_core->>platform: sign(pinSignedInstruction)
            platform->>platform: signWithHwKey(pinSignedInstruction)
            platform-->>wallet_core: doubleSignedInstruction
        end
        wallet_core->>wallet_provider: instruction(doubleSignedInstruction)
        opt server error
            Note over wallet_provider,user: same as server error above
        end
        wallet_provider-->>wallet_core: result
        alt error
            wallet_core-->>wallet_app: pin error
            Note over wallet_core,wallet_app: IncorrectPin,<br/>Timeout, Blocked
            wallet_app->>user: render error & request new pin
        else success
            wallet_core-->>wallet_app: success
            wallet_app->>user: render unlocked
        end
    deactivate user
```
