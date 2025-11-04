# NL Wallet information assets

## Overview 

### Cryptographic keys (Assets)

| Cryptographic Key                                    | Alg./ Size           | Store                             | Usage                                                  |
| ---------------------------------------------------- | -------------------- | --------------------------------- | ------------------------------------------------------ |
| HW Private Key                                       | ECDSA (nistP256)     | Wallet Secure Element             | Signing of instructions (from Wallet App)              |
| PIN Salt                                             | 32 bytes             | Wallet App Database               | Salt for user PIN                                      |
| Wallet Provider Instruction Signing Key              | ECDSA (nistP256)     | HSM                               | Signing of instruction responses (from Wallet Backend) |
| Wallet Provider Wallet Certificate Signing Key       | ECDSA (nistP256)     | HSM                               | Signing of Wallet Certificate                          |
| Wallet Provider Attestation Wrapping Key (WPASWP)    | AES 256 GCM (?)      | HSM                               | Encryption of Credential Private keys                  |
| Wallet Provider PIN Encryption Key                   | AES 256 GCM          | HSM                               | ?                                                      |
| Wallet Provider PIN Public Disclosure Protection Key | HMAC SHA256 64 bytes | HSM                               | ?                                                      |
| Wallet Provider WUA Signing Key                      | ECDSA (nistP256)     | HSM                               | Signing of WUA                                         |
| Credential Private Key                               | ECDSA (nistP256)     | WP Database (encrypted by WPASWP) | Signing of PoP's                                       |
| Wallet Provider TLS Private Key                      |                      | WP Backend/Ingress Keystore (?)   | Transport security                                     |


### Other information assets
| Asset                                    | Format             | Store                            | Issuer          | Usage                                       |
| ---------------------------------------- | ------------------ | -------------------------------- | --------------- | ------------------------------------------- |
| PID data                                 | SD-JWT /<br/> mdoc | Wallet app database              | PID Issuer      | Disclosure of data to RP's and Issuers      |
| Credential data (other credential types) | SD-JWT /<br/> mdoc | Wallet app database              | Other issuers   | Disclosure of data to RP's and Issuers      |
| Wallet Certificate                       | JWT                | Wallet App Database              | Wallet Provider | Authenticating Wallet to WP Wallet Provider |
| Wallet Unit Attestation                  | SD-JWT             | Transient (passed to PID Issuer) | Wallet Provider | Authenticating Wallet Unit to PID Issuer    |


### Cryptographic keys (Supporting)
| Cryptographic Key              | Alg./ Size | Store | Usage                |
| ------------------------------ | ---------- | ----- | -------------------- |
| Wallet database encryption key |            | ?     | Wallet DB encryption |

