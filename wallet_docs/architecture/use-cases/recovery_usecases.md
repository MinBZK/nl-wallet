# Wallet recovery use cases

1. [PIN Recovery](#pin-recovery): 
With the PIN Recovery flow, the user can reset a forgotten PIN by reauhtenticating with DigiD. 

2. [PID renewal flow](#pid-renewal-flow):
PID Renewal allows the user to fetch a new PID attestation in case the current PID is expired or revoked.

3. [Wallet device transfer](#wallet-device-transfer)
Wallet Device Transfer will allow transfer of contents from an existing active wallet to a newly activated wallet on another device.

## PIN Recovery

PIN Recovery is desired in the following situations
1. When the user has entered his PIN wrongly (> max attempts). In this case the Wallet is blocked from further usage.
2. When the user forgot his PIN. 

To restore access to his Wallet, the user can use the PIN-Recovery flow. In this flow, the user will:
1. Select a new PIN
2. Have to reauthenticate using DigiD
3. When user identity matches with the registered user identity, unblock the account and use the new PIN

PIN Recovery flow reuses parts of the [PID issuance flow](./issuance-with-openid4vci.md).

### PIN Recovery flow sequence diagram

The sequence diagram below, describes the process and the interactions between the relevant components. 

```{mermaid}
sequenceDiagram
    autonumber
    actor User as User
    participant Wallet as Wallet App
    participant WP as Wallet Backend
    participant PID as PID Issuer
    
    note over User, WP: Account status is 'blocked' or user forgot PIN 
    User ->> Wallet : Start PIN Recovery 
    note over Wallet, PID:Steps 2-17 from PID-issuance flow. <br/> Wallet has received access token from PID Issuer (and PID-preview)
    Wallet ->> Wallet: Check Recovery code in stored PID against recovery code in PID-preview    
    Wallet ->> User: Request new PIN
    User ->> Wallet: Give new PIN (including confirmation)
    Wallet ->> WP: Send start_pin_recovery(new PIN, c_nonce for PID issuance)
    WP ->> WP: Update PIN for account<br/> sign PIN cert<br/>genereate new keys (marked as recovery keys)<br/> sign PoP's<br/> issue WUA<br/>set account in 'recovery' state
    WP ->> Wallet: New PIN OK, return new PIN certificate, WUA, signed PoP's
    Wallet ->> Wallet: Delete previous PIN data, store new PIN data    
    note over Wallet, PID: Steps 22-24 from PID-issuance flow. <br/> Wallet has requested and received attestations from PID-issuer. 
    Wallet ->> WP: Disclose Recovery code from new PID-attestation
    WP ->> WP: Verify recovery code with account (must match)<br/>Remove private keys that were created for PID-attestation used for recovery<br/>Update account to 'active'
    WP ->> Wallet: Report PIN Recovery success
    Wallet ->> Wallet: Dispose PID used in PIN-recovery process
    Wallet ->> User: Show PIN Recovery complete
```



## PID renewal flow

PID renewal flow reuses steps from [PID issuance flow](./issuance-with-openid4vci.md) 

```{mermaid}
sequenceDiagram
    autonumber
    actor User as User
    participant OS as iOS/Android
    participant Wallet as Wallet App
    participant WP as Wallet Provider
    participant PID as PID Issuer
    
    note over User, Wallet: Wallet is active, PID is revoked/expired' 
    User ->> Wallet : Start PID-reissuance
    note over Wallet, PID:Steps 2-17 from DigiD authentication / PID-issuance flow. <br/> Wallet has received access token from PID Issuer (and PID-preview)
    Wallet ->> Wallet: Check Recovery code in stored PID against recovery code in PID-preview    
    note over Wallet, PID: Steps 20-23 from PID-issuance flow. <br/> Wallet has requested and received attestations from PID-issuer. WP creates PoPs and new WUA. <br/>Issuance of new WUA will put account into 'recovery' and disable existing keys except the newly created keys for PID issuance.
    Wallet ->> WP: Disclose recovery code from new PID to WP
    WP ->> WP: Check recovery code with account (fail if not matching) and update account to 'active'
    WP ->> Wallet: New PID accepted
    Wallet ->> Wallet: Store new PID attestation
    Wallet ->> User: Report success (New PID visible in wallet)
```


## Wallet device transfer 

Wallet device transfer allows the user the move te contents of a wallet installation on a source device to another (destination) device.

### Transfer flow
```{mermaid}
sequenceDiagram
    autonumber

    actor User
    participant WT as Destination Wallet
    participant WS as Source Wallet
    participant WP as Wallet Backend (WP)

    User ->> WT: Yes start migration (using `transfer_session_id` resulting from recovery_code disclosure in activation) (tranfer_state= created)
    WT ->> WT: Create asymmetric EC key (using ECIES)
    WT ->> User: Present QR code containing pubkey and transfer_session_id (to be scanned from source device)
    User ->> WS: Open app (with PIN/biometrics), Scan QR from target device (read public key and transfer_session_id)
    WS ->> WP: confirm_transfer_session (with transfer_session_id, app_version)
    WP ->> WP: validate recovery codes for Source and Destination Wallets (must be equal)
    WP ->> WP: validate app versions (Destination app_version must be >= than than Source app_version) <br/>tranfer_state = ready_for_transfer
    WP ->> WS: confirm session (ok)
    WS ->> User: Request PIN for data transfer (final warning)
    User ->> WS: Confirm transfer with PIN
    WS ->> WS: Encrypt database to encrypted_payload
    WS ->> WP: send_wallet_payload (transfer_session_id, encrypted_payload)
    WP ->> WP: stash payload for Destination Wallet<br/>transfer_state = ready_for_download
    note over WT, WP: Poll for payload while not cancelled.
    WT ->> WP: receive_payload (transfer_session_id)
    WP ->> WP: while not (ready_for_download) complete from source wallet: return pending else return payload
    note over WS, WP: Poll transfer status while transfer not completed or cancelled.
    WS ->> WP: check_transfer_status(transfer_session_id)  
    WP ->> WS: return transfer status (pending, cancelled, completed)  
    WP ->> WT: Transfer encrypted_payload to Destination Wallet
    WT ->> WT: decrypt data, restore wallet
    WT ->> WP: complete_transfer(transfer_session_id) 
    WP ->> WP: move private keys from Source Wallet to Destination Wallet<br/>transfer_session = complete<br/>source wallet state=transferred
    WP ->> WT: Report session complete
    WT ->> WT: set imported database as current database<br/>dispose database that was used in onboarding
    WT ->> User: Transfer complete
    WS ->> User: transfer complete (Source wallet now deactivated)
   
```
### Data encryption

Data exchanged in Wallet Device transfer is encrypted, using ECIES in JWE. Encryption/decryption of data is performed in the following steps from the above Sequence Diagram:

2) The public/private key pair is generated in Step 2. 

4. The public key is exchanged from the destination device to the source device using the presented QR-code (step 4). 

11) The transfered data (step 11) is encoded in a JWE using ECIES:  ECDH-ES (for key agreement) + symmetric encryption (AES-GCM).

19. In step 19, the encrypted data is retrieved from the WalletBackend and will be decrypted in the Destionation Wallet.

### Wallet states during transfer

While transfering, the following states are used in the process:

| Transfer State             | Will be set by                                                        | Next State (s)                                |
|----------------------------|-----------------------------------------------------------------------|-----------------------------------------------|
| created                    | After activation of DW, when another account exists for the same user | ready_for_transfer
| ready_for_transfer         | After scan QR from DW. SW is linked to transfer session.              | ready_for_download, cancelled 
| ready_for_download         | After upload payload from SW.                                         | completed, cancelled
| completed                  | DW after succesfull download and processing of payload                | -                    
| cancelled                  | User can cancel transfer from DW or SW                                | -


