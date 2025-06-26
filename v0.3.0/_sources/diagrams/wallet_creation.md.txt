# Wallet creation

The diagram below illustrates the Wallet creation process. Including certificate generation and registration with the `wallet_provider`.

## Create wallet (app)

```{mermaid}
sequenceDiagram
    %% Force ordering by explicitly setting up participants
    actor user
    participant platform
    participant wallet_app
    participant wallet_core
    participant wallet_provider
    title Create Wallet (app) [2.1]

    user->>wallet_app: provide valid pin
    note over user, wallet_app: pin validity is guaranteed locally, see 3.1
    activate user
        wallet_app->>wallet_core: register(pin)
        wallet_core->>wallet_provider: requestChallenge()
        opt server error
        wallet_provider-->>wallet_core: server error
            wallet_core-->>wallet_app: error
            wallet_app->>user: show server error
        end
        note over wallet_provider,wallet_provider: see 2.2 for details
        wallet_provider-->>wallet_core: challenge
        activate wallet_core
        critical key generation
            option key setup (pin key)
                wallet_core->>wallet_core: generateSalt()
                wallet_core->>platform: store(salt)
                platform->>platform: generateHwBackedDbKey()
                platform->>platform: persistEncrypted(salt)
            option key setup (hw key)
                wallet_core->>platform: generateHwKey()
                note over platform, wallet_core: HW key = Hardware backed key
        end
        critical sign challenge
            option sign with pin key
                wallet_core->>wallet_core: deriveEcdsaKeyPair(pin, salt)
                wallet_core->>wallet_core: signChallenge(ecdsaKey, challenge)
            option sign with hw key
                wallet_core->>platform: signWithHwKey(pinSignedChallenge)
                platform->>platform: sign(hwKey, pinSignedChallenge)
                platform-->>wallet_core: doubleSignedChallenge
        end
        wallet_core->>platform: getHwPublicKey()
        platform-->>wallet_core: hwPublicKey
        wallet_core->>+wallet_provider: createWallet(<br/>pinPubKey, hwPubKey<br/>doubleSignedChallenge<br/>)
        deactivate wallet_core
        wallet_provider->>wallet_provider: createWallet()
        opt server error
            note over wallet_provider,user: same as server error above
        end
        note over wallet_provider,wallet_provider: see 2.2 for details
        wallet_provider-->>-wallet_core: walletId, walletCertificate
        wallet_core->>platform: store(walletId, walletCert)
        platform->>platform: persist(walletId, walletCert)
        wallet_core->>wallet_app: registration success
        wallet_app->>user: render wallet created
    deactivate user
```

## Create wallet (server)

High level overview of what happens inside the `wallet_provider`.

```{mermaid}
sequenceDiagram
    %% Force ordering by explicitly setting up participants
    actor user
    participant wallet_core
    participant wallet_provider
    participant hsm
    title Create Wallet (server) [2.2]

    wallet_core->>+wallet_provider: requestChallenge() 
    note over wallet_core,wallet_provider: POST /enroll
    wallet_provider->>wallet_provider: generateChallenge()
    wallet_provider-->>-wallet_core: challenge
    wallet_core->>+wallet_provider: createWallet(<br/>pinPubKey, pinSignedChallenge<br/>hWPubKey, hwSignedChallenge<br/>)
    note over wallet_core,wallet_provider: POST /createwallet
    wallet_provider->>wallet_provider: verifySignedChallenge()
    wallet_provider->>wallet_provider: storePubKeys()
    wallet_provider->>wallet_provider: generateCertificate()
    wallet_provider-->>-wallet_core: walletId, walletCertificate
```
