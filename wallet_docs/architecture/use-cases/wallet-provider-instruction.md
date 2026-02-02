# Wallet Provider instructions

After the wallet has enrolled to the Wallet Backend (WB), it communicates with it by sending it instructions.
The wallet and WB support an instruction for each task that the wallet needs the WB to do.
The instruction signing process includes authenticating the wallet and user by the hardware bound private key and the PIN. 
The following sequence diagram depicts this process.

``` {mermaid}
sequenceDiagram
    actor user
    participant platform as Mobile Platform<br/> (SE/TEE)
    participant db as App Database
    participant wallet as Wallet App
    participant wallet_provider as Wallet Backend
    participant wp_db as WB Database
    title Send Wallet Backend Instruction

    user->>wallet: enter PIN

    activate wallet
        wallet->>+platform: signWithHwKey({})
        platform-->>-wallet: signedChallengeRequest
        
        wallet->>+wallet_provider: requestChallenge(signedChallengeRequest, WbCertificate)
        note over wallet,wallet_provider: POST /instructions/challenge
        wallet_provider ->> wallet_provider: walletHwBoundPublicKey, walletID = verify(WbCertificate, WalletCertificateSigningPublicKey)
        wallet_provider ->> wallet_provider: verify(signedChallengeRequest, walletHwBoundPublicKey)
        wallet_provider ->> wallet_provider: generate random challenge
        wallet_provider->>+wp_db: storeChallenge(walletID)
        wp_db-->>-wallet_provider: OK
        wallet_provider-->>-wallet: challenge
        
        wallet->>+db: retrieveAndDecryptSalt()
        db-->>-wallet: salt
        wallet->>wallet: deriveWalletPINPrivateKey(pin, salt)
        wallet->>wallet: instruction = {challenge, instruction_specific_data}<br/>pinSignedInstruction = sign(walletPINPrivateKey, instruction)
        wallet->>+platform: signWithHwKey(pinSignedInstruction)
        platform-->>-wallet: doubleSignedInstruction
        
        wallet->>+wallet_provider: instruction(doubleSignedInstruction, WbCertificate)
        note over wallet,wallet_provider: POST /instructions/[instruction_name]
        
        wallet_provider->>wallet_provider: verify instruction
        wallet_provider->>wallet_provider: execute instruction
        wallet_provider->>wallet_provider: sign instruction response
        
        wallet_provider-->>-wallet: signedResponse
        wallet->>wallet: verify(signedResponse, instructionSigningPublicKey)
        wallet->>wallet: handle response
    deactivate wallet
```

The instruction verification and execution process in the Wallet Backend is detailed below.

``` {mermaid}
sequenceDiagram
    participant wallet as Wallet App
    participant wallet_provider as Wallet Backend
    participant hsm as WB HSM
    participant wp_db as WB Database
    title Verify Wallet Backend Instruction

    wallet->>+wallet_provider: instruction(doubleSignedInstruction, WbCertificate)
    note over wallet,wallet_provider: POST /instructions/[instruction_name]
    wallet_provider->>wallet_provider: walletHwBoundPublicKey, walletID = verify(WbCertificate, WalletCertificateSigningPublicKey)
    wallet_provider->>+wp_db: retrieveUser(walletID)
    wp_db-->>-wallet_provider: user
    wallet_provider->>wallet_provider: check that walletHwBoundPublicKey == user.walletHwBoundPublicKey
    wallet_provider->>+hsm: decrypt(user.encryptedPINPublicKey, PINPublicKeyEncryptionKey)
    hsm-->>-wallet_provider: walletPINPublicKey
    wallet_provider->>+hsm: HMAC(walletPINPublicKey, PINHMACKey)
    hsm-->>-wallet_provider: PINPublicKeyHMAC
    wallet_provider->>wallet_provider: check that PINPublicKeyHMAC == WbCertificate.PINPublicKeyHMAC
    wallet_provider->>wallet_provider: pinSignedInstruction = verifyHwSignature(doubleSignedInstruction, walletHwBoundPublicKey)
    wallet_provider->>wallet_provider: instruction = verifyPINSignature(pinSignedInstruction, walletPINPublicKey)
    wallet_provider->>+wp_db: retrieveChallenge(walletID)
    wp_db-->>-wallet_provider: challenge
    wallet_provider->>+wp_db: clearChallenge(walletID)
    wp_db-->>-wallet_provider: OK
    wallet_provider->>wallet_provider: check that instruction.challenge == challenge
    wallet_provider->>wallet_provider: check that instruction.sequenceNumber > user.sequenceNumber
    wallet_provider->>+wp_db: set user.sequenceNumber to instruction.sequenceNumber
    wp_db-->>-wallet_provider: OK
    wallet_provider->>wallet_provider: response = handleInstruction(instruction)
    wallet_provider->>+hsm: sign(response, instructionSigningPrivateKey)
    hsm-->>-wallet_provider: signedResponse
    
    wallet_provider-->>-wallet: signedResponse
```