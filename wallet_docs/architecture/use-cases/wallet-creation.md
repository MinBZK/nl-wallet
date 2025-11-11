# Wallet Creation

The diagram below illustrates the Wallet creation process. Including certificate generation and registration with the `wallet_provider`.

## Create wallet (app)

```{mermaid}
sequenceDiagram
    actor user
    participant platform as Platform<br/> (SE/TEE)
    participant db as App Database
    participant wallet_app as Wallet Frontend (App)
    participant wallet_core as Wallet Core (App)
    participant wallet_provider as Wallet Backend
    title Create Wallet (app) [2.1]

    user->>wallet_app: provide valid pin
    activate user
        wallet_app->>wallet_core: register(pin)
        activate wallet_core
        wallet_core->>+wallet_provider: POST /enroll<br/> requestChallenge()
        wallet_provider-->>-wallet_core: challenge
        wallet_core->>wallet_core: generateSalt()
        wallet_core->>db: storeEncrypted(salt)
        wallet_core->>wallet_core: deriveWalletPINKeyPair(pin, salt)
        wallet_core->>wallet_core: signChallenge(walletPINPrivateKey, challenge)
        wallet_core->>platform: generateHwKey()
        wallet_core->>+platform: signWithHwKey(pinSignedChallenge)
        platform->>platform: sign(hwKey, pinSignedChallenge)
        platform-->>-wallet_core: doubleSignedChallenge
        wallet_core->>+platform: getHwPublicKey()
        platform-->>-wallet_core: hwPublicKey
        wallet_core->>+wallet_provider: POST /createWallet ,<br/>createWallet(<br/>walletPINPubKey, hwPubKey<br/>doubleSignedChallenge,<br/> appAttestation, keyAttestation)
        wallet_provider-->>-wallet_core: WBCertificate
        wallet_core->>db: storeEncrypted(WBCertificate)
        wallet_core->>wallet_app: registration success
        deactivate wallet_core
        wallet_app->>user: render wallet created
    deactivate user
```

## Create wallet (Wallet Backend)

High level overview of what happens inside the `Wallet Bakckend`.

```{mermaid}
sequenceDiagram
    %% Force ordering by explicitly setting up participants
    participant wallet_core as Wallet Core (App)
    participant wallet_provider as Wallet Backend
    participant hsm as HSM
    participant db as WB Database
    title Create Wallet (server) [2.2]

    wallet_core->>+wallet_provider: requestChallenge()
    note over wallet_core,wallet_provider: POST /enroll
    wallet_provider->>wallet_provider: generateChallengePayload()
    wallet_provider ->> hsm: Sign challengePayload using WB Wallet Certificate Signing Key 
    hsm ->> wallet_provider: signedChallenge
    wallet_provider-->>-wallet_core: signedChallenge (JWT)
    wallet_core->>+wallet_provider: createWallet(<br/>walletPINPubKey, doubleSignedChallenge<br/>hwPublicKey, appAttestation, keyAttestation)
    note over wallet_core,wallet_provider: POST /createwallet
    wallet_provider->>wallet_provider: verifyDoubleSignedChallenge()
    wallet_provider->>wallet_provider: verifyAppKeyAttestation(appAttestation, keyAttestation)
    wallet_provider->>db: storePubKeys(), storeAppAttestation(), storeKeyAttestation()
    wallet_provider->>wallet_provider: generateWBCertificateContent()
    wallet_provider->> hsm: sign WBCertificate content using walletCertificateSigningKey 
    hsm ->> wallet_provider:  WBCertificate
    wallet_provider-->>-wallet_core: WBCertificate
```
