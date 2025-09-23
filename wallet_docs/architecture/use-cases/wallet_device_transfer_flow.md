# Wallet device transfer 

Wallet device transfer allows the user the move te contents of a wallet installation on a source device to another (destination) device.

## Transfer flow
```{mermaid}
sequenceDiagram
    autonumber

    actor User
    participant WT as Destination Wallet
    participant WS as Source Wallet
    participant WP as Wallet Backend (WP)

    User ->> WT: Yes start migration (using `transfer_session_id` resulting from recovery_code disclosure in activation)
    WT ->> WP: prepare_transfer(transfer_session_id, app_version)
    WP ->> WP: check if migration possible
    WP ->> WP: put Destination Wallet in 'transfering_to' state
    WP ->> WT: return session id 
    WT ->> WT: Create asymmetric EC key (using ECIES)
    WT ->> User: Present QR code containing pubkey and transfer_session_id (to be scanned from source device)
    User ->> WS: Open app (with PIN/biometrics), Scan QR from target device (read public key and transfer_session_id)
    WS ->> WP: confirm_transfer session (with transfer_session_id, app_version)
    WP ->> WP: validate recovery codes for Source and Destination Wallets (must be equal)
    WP ->> WP: validate app versions (Destination app_version must be >= than than Source app_version)
    WP ->> WS: confirm session
    WS ->> User: Request PIN for data transfer (final warning)
    User ->> WS: Confirm transfer with PIN
    WS ->> WS: Encrypt database to encrypted_payload
    WS ->> WP: send_wallet_payload (transfer_session_id, encrypted_payload)
    WP ->> WP: put wallet in 'transfering_from' state, stash payload for Destination Wallet
    note over WT, WP: Poll for payload while not cancelled.
    WT ->> WP: receive_payload (transfer_session_id)
    WP ->> WP: while not (send_payload) complete from source wallet: return pending else return payload
    WP ->> WT: Transfer encrypted_payload to WT
    WT ->> WT: decrypt data
    WT ->> WP: complete_transfer(transfer_session_id) 
    WT ->> User: Transfer complete
    note over WS, WP: Poll transfer status while transfer not completed or cancelled.
    WS ->> WP: check_transfer_status(transfer_session_id)  
    WP ->> WS: return transfer status (pending, cancelled, completed)  
    WS ->> User: transfer complete (Source wallet now deactivated)
   
```
## Data encryption

Data exchanged in Wallet Device transfer is encrypted, using ECIES in JWE. Encryption/decryption of data is performed in the following steps from the above Sequence Diagram:

6) The public/private key pair is generated in Step 6. 

7. The public key is exchanged from the destination device to the source device using the presented QR-code (step 7). 

15) The transfered data (step 15) is encoded in a JWE using ECIES:  ECDH-ES (for key agreement) + symmetric encryption (AES-GCM).

20. In step 20, the encrypted data is retrieved from the WalletBackend and will be decrypted in the Destionation Wallet.