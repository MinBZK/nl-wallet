# Wallet recovery

## Recovery code

When a user loses their PIN or device or when they wish to migrate their wallet data to another device within the same wallet solution, the restored or new account must be matched to the original account with a high level of assurance. 

The NL PID Provider includes a persistent Recovery Code as part of the PID that is issued to the NL Wallet in the onboarding process. This Recovery Code is derived from the BSN and shall be consistent across all issuances of PID of the same user to the NL Wallet. 
The Recovery Code will be stored in the user account (in Wallet Backend, not on user device) after PID issuance.

The Recovery Code is solely intended to be used by the NL Wallet and will be prevented from being disclosed to Relying Parties.

The Recovery Code is used in the usecases described below:

## Recovery usecases
NL Wallet supports the following use cases that rely on the Recovery Code: 

1. [PIN Recovery](#pin-recovery): 
With the PIN Recovery flow, the user can reset a forgotten PIN by re-authenticating with DigiD. 

3. [Wallet device transfer](#wallet-device-transfer)
Wallet Device Transfer will allow transfer of contents from an existing active wallet to a newly activated wallet on another device.

3. [PID renewal flow](#pid-renewal):
PID Renewal allows the user to fetch a new PID attestation in case the current PID is expired or revoked.


### PIN Recovery

PIN Recovery is desired in the following situations
1. When the user has entered his PIN wrongly (> max attempts). In this case the Wallet is blocked from further usage.
2. When the user forgot his PIN. 

To restore access to his Wallet, the user can use the PIN-Recovery flow. In this flow, the user will:
1. Select a new PIN
2. Reauthenticate using DigiD
3. When user identity matches with the registered user identity (has same Recovery Code), unblock the account and use the new PIN

PIN Recovery flow reuses parts of the [PID issuance flow](./issuance-with-openid4vci.md).

#### PIN Recovery flow sequence diagram

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
    note over User, PID:Steps 2-17 from PID-issuance flow. <br/> Wallet has received access token from PID Issuer (and PID-preview)
    Wallet ->> Wallet: Check Recovery code in stored PID against recovery code in PID-preview<br/>Must match, else terminate with error
    Wallet ->> User: Request new PIN
    User ->> Wallet: Give new PIN (including confirmation)
    Wallet ->> WP: Send start_pin_recovery(new PIN public key, c_nonce for PID issuance)
    WP ->> WP: Update PIN public key for account<br/> sign Wallet Certificate<br/>genereate new keys (blocked for regular use)<br/> sign PoP's<br/> issue WUA<br/>set account in 'recovery' state
    WP ->> Wallet: New PIN OK, return new Wallet Certificate, WUA, signed PoP's
    Wallet ->> Wallet: Delete previous Wallet Certificate,<br/> store new Wallet Certificate    
    note over Wallet, PID: Steps 22-24 from PID-issuance flow. <br/> Wallet has requested and received attestations from PID-issuer. 
    Wallet ->> WP: Send DiscloseRecoveryCodePinRecovery(new PID, with recovery code) instruction
    WP ->> WP: Verify recovery code with account (must match)<br/>Remove private keys that were created at start_pin_recovery<br/>Update account to 'active'
    WP ->> Wallet: Report PIN Recovery success
    Wallet ->> Wallet: Dispose PID used in PIN-recovery process
    Wallet ->> User: Show PIN Recovery complete
```



### Wallet device transfer 

Wallet device transfer allows the user the move te contents of a wallet installation on a source device to another device.

To migrate a wallet from a source device to another destination device, the user can use Wallet Device Transfer flow. In this flow, the user will:
1. Active a new NL Wallet instance on the destination device (Destination Wallet) using the regular onboarding flow. After the onboarding flow, the Recovery Code (from the PID) wil be disclosed to NL Wallet. If the NL Wallet discovers that another account is active (has the same Recovery Code) a migration possibility will be offered to the user.
2. Transfer the content of his Source Wallet to the Destination Wallet. This is only possible when the identity of the Destination Wallet matches with the registered user identity in the Source Wallet account (has same Recovery Code)
3. After completion, the contents of the Source Wallet are migrated to the Destination Wallet. The Source Wallet will be emptied.


#### Wallet Device Transfer flow

The sequence diagram below, describes the process and the interactions between the relevant components. 

```{mermaid}
sequenceDiagram
    autonumber

    actor User
    participant WT as Destination Wallet
    participant WS as Source Wallet
    participant WP as Wallet Backend

    User ->> WT: Yes start migration (using `transfer_session_id` resulting from recovery_code disclosure in activation) <br/>(tranfer_state= created)
    WT ->> WT: Create asymmetric EC key (using ECIES)
    WT ->> User: Present QR code containing pubkey and transfer_session_id (to be scanned from source device)
    User ->> WS: Open app (with PIN/biometrics), Scan QR from target device (read public key and transfer_session_id)
    WS ->> WP: confirm_transfer_session (with transfer_session_id, app_version)
    WP ->> WP: validate recovery codes for Source and Destination Wallets (must be equal)
    WP ->> WP: validate app versions (Destination app_version must be >= than than Source app_version) <br/>tranfer_state = ready_for_transfer
    WP ->> WS: confirm session (ok)
    WS ->> User: Request PIN to confirm data transfer
    User ->> WS: Confirm transfer with PIN
    WS ->> WS: Encrypt database to encrypted_payload
    par Upload payload from Source Wallet and wait for completion.
        WS ->> WP: send_wallet_payload (transfer_session_id, encrypted_payload)
        WP ->> WP: stash payload for Destination Wallet<br/>transfer_state = ready_for_download
        WP ->> WS: ok
        note over WS, WP: After upload: Poll transfer status while transfer not completed or canceled.
        WS ->> WP: check_transfer_status(transfer_session_id)  
        WP ->> WS: return transfer status (pending, canceled, completed)  
    and (Wait for and) Receive payload from Wallet Backend and report migration status back to Wallet Bakckend afterwards.
        WT ->> WP: receive_payload (transfer_session_id)
        WP ->> WP: while not (ready_for_download) complete from source wallet: <br/>return pending else return payload
        WP ->> WT: Transfer encrypted_payload to Destination Wallet
        WT ->> WT: decrypt data, restore wallet
        WT ->> WP: complete_transfer(transfer_session_id) 
        WP ->> WP: move private keys from Source Wallet to Destination Wallet<br/>transfer_session = complete<br/>source wallet state=transferred
        WP ->> WT: Report session complete
    end
    WT ->> WT: set imported database as current database<br/>sync instruction counter (from previous database)<br/>dispose previous database that was used in onboarding
    WT ->> User: Transfer complete
    WS ->> User: transfer complete (Source wallet now deactivated)
   
```
#### Data encryption

Data exchanged in Wallet Device transfer is encrypted, using ECIES in JWE. Encryption/decryption of data is performed in the following steps from the above Sequence Diagram:

2) The public/private key pair is generated in Step 2. 

4. The public key is exchanged from the destination device to the source device using the presented QR-code (step 4). 

11) The data (step 11) is encoded in a JWE using ECIES:  ECDH-ES (for key agreement) + symmetric encryption (AES-GCM).

19. In step 19, the encrypted data is retrieved from the WalletBackend and will be decrypted in the Destionation Wallet.

#### Wallet states during transfer

While transfering, the following states are used in the process:

| Transfer State             | State is set when                                                                     | Next State(s)                                 |
|----------------------------|---------------------------------------------------------------------------------------|-----------------------------------------------|
| created                    | After activation of Destination Wallet, when another account exists for the same user | ready_for_transfer, canceled
| ready_for_transfer         | After scanning QR from Destination Wallet and `confirm_transfer_session` instruction, Source Wallet is linked to transfer session.                              | ready_for_download, canceled 
| ready_for_download         | After `send_wallet_payload` instruction from Source Wallet                            | completed, canceled
| completed                  | Destination Wallet after succesfull download (`receive_wallet_payload` instruction ) <br/>and processing of payload confirmed with `complete_transfer`                                | -                    
| canceled                   | User can cancel transfer from Destination Wallet or Source Wallet from any state using `cancel_transfer` instruction. <br/>Not allowed after `completed` state is reached                      | -



### PID Renewal

PID Renewal allows the user to fetch a new PID attestation in case the current PID is expired or revoked.

The PID renewal flow reuses steps from [PID issuance flow](./issuance-with-openid4vci.md) 


#### PID Renewal flow sequence diagram


```{mermaid}
sequenceDiagram
    autonumber
    actor User as User
    participant Wallet as Wallet App
    participant WP as Wallet Backend
    participant PID as PID Issuer
    
    note over User, Wallet: Wallet is active, PID is revoked/expired' 
    User ->> Wallet : Start PID renewal
    note over Wallet, PID:Steps 2-17 from DigiD authentication / PID-issuance flow. <br/> Wallet has received access token from PID Issuer (and PID-preview)
    Wallet ->> Wallet: Check Recovery code in stored PID against recovery code in PID-preview    
    note over Wallet, PID: Steps 20-23 from PID-issuance flow. <br/> Wallet has requested and received attestations from PID-issuer. Wallet Backend creates PoPs and new WUA. <br/>Issuance of new WUA will put account into 'recovery' and disable existing keys except the newly created keys for PID issuance.
    Wallet ->> WP: Call DiscloseRecoveryCode(recovery_code) (Recovery Code from new PID is used) to Wallet Backend.
    WP ->> WP: Check recovery code with account (fail if not matching) <br/> and update account to 'active'
    WP ->> Wallet: New PID accepted
    Wallet ->> Wallet: Store new PID attestation (as replacement for previous PID)
    Wallet ->> User: Report success (New PID visible in wallet)
```

