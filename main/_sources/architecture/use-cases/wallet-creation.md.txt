# Wallet Creation

The diagram below illustrates the Wallet creation process, including certificate generation and registration with the Wallet Backend.

## Create wallet

```{mermaid}
sequenceDiagram
    actor user
    participant platform as Mobile Platform<br/> (SE/TEE)
    participant db as App Database
    participant wallet_core as Wallet App
    participant wallet_provider as Wallet Backend
    participant hsm as WB HSM
    participant wp_db as WB Database
    title Create Wallet

    user->>wallet_core: provide valid pin
    
    activate wallet_core
        wallet_core->>+wallet_provider: requestChallenge()
        
        note over wallet_core,wallet_provider: POST /enroll
        wallet_provider->>wallet_provider: generate new walletID
        wallet_provider ->>+ hsm: sign(walletID, WalletCertificateSigningPrivateKey)
        hsm -->>- wallet_provider: challengeJWT
        wallet_provider-->>-wallet_core: challengeJWT
        
        wallet_core->>wallet_core: generateSalt()
        wallet_core->>+db: storeEncrypted(salt)
        db-->>-wallet_core: OK
        wallet_core->>wallet_core: walletPINPrivateKey, walletPINPublicKey = deriveWalletPINKeyPair(pin, salt)
        wallet_core->>+platform: generateHwPrivateKey()
        platform-->>-wallet_core: walletHwBoundPublicKey
        wallet_core->>wallet_core: perform key & app attestation
        wallet_core->>wallet_core: pinSignedRegistrationMessage = sign({challengeJWT, walletPINPublicKey, walletHwBoundPublicKey, keyAttestation, appAttestation}, walletPINPrivateKey)
        wallet_core->>+platform: signWithHwKey(pinSignedRegistrationMessage)
        platform-->>-wallet_core: doubleSignedRegistrationMessage
        
        wallet_core->>+wallet_provider: createWallet(doubleSignedRegistrationMessage)
        note over wallet_core,wallet_provider: POST /createwallet
        wallet_provider->>wallet_provider: walletHwBoundPublicKey, walletPINPublicKey = parsePinAndHwKeys(doubleSignedRegistrationMessage)
        wallet_provider->>wallet_provider: pinSignedRegistrationMessage = verifyHwSignature(doubleSignedRegistrationMessage, walletHwBoundPublicKey)
        wallet_provider->>wallet_provider: registrationMessage = verifyPINSignature(pinSignedRegistrationMessage, walletPINPublicKey)
        wallet_provider->>wallet_provider: verify(registrationMessage.challengeJWT, WalletCertificateSigningPublicKey)
        wallet_provider->>wallet_provider: verifyAppKeyAttestation(registrationMessage.appAttestation, registrationMessage.keyAttestation)
        wallet_provider->>+hsm: encrypt(walletPINPublicKey, PINPublicKeyEncryptionKey)
        hsm->>-wallet_provider: encryptedPINPublicKey
        wallet_provider->>+wp_db: storeNewUser(walletID, encryptedPINPublicKey, walletHwBoundPublicKey, keyAttestation, appAttestation)
        wp_db-->>-wallet_provider: OK
        wallet_provider->>+hsm: HMAC(walletPINPublicKey, PINHMACKey)
        hsm-->>-wallet_provider: PINPublicKeyHMAC
        wallet_provider->>+ hsm: sign({walletID, walletHwBoundPublicKey, PINPublicKeyHMAC}, walletCertificateSigningKey)
        hsm ->>- wallet_provider: WBCertificate
        wallet_provider-->>-wallet_core: WBCertificate
        
        wallet_core->>+db: storeEncrypted(WBCertificate)
        db-->>-wallet_core: OK
    deactivate wallet_core
```
