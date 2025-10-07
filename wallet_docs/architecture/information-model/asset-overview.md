# NL Wallet information assets

## Overview 

### Cryptographic keys (Assets)

| Cryptographic Key                                | Alg./ Size          | Store                  | Usage
|--------------------------------------------------|---------------------|------------------------|----------------|
| HW Private Key                                   |                     | Wallet Secure Element  | Signing of instructions (from Wallet App)
| PIN Private Key                                  |                     | Wallet Secure Element  | Signing of instructions (from Wallet App)
| Wallet Provider Instruction Signing Key          |                     | HSM                    | Signing of instruction responses (from Wallet Backend)
| Wallet Provider Wallet Certificate Signing Key   |                     | HSM                    | Signing of Wallet Certificate
| Wallet Provider Attestation Wrapping Key (WPASWP)|                     | HSM                    | Encryption of Credential Private keys 
| Wallet Provider Encryption Key                   |                     | HSM                    | ?
| Wallet Provider Public Disclosure Protection Key |                     | HSM                    | ?
| Wallet Provider WUA Signing Key                  | ECDSA 256 (nistP256)| HSM                    | Signing of WUA 
| Credential Private Key                           |                     | WP Database (encrypted by WPASWP)           | Signing of PoP's
| Wallet Provider TLS Private Key                  |                     | WP Backend/Ingress Keystore (?) | Transport security


### Other information assets
| Asset                                      | Format             | Store                  | Issuer         | Usage
|--------------------------------------------|--------------------|------------------------|----------------|-----------|
| PID data                                   | SD-JWT /<br/> mdoc | Wallet app database    | PID Issuer | Disclosure of data to RP's and Issuers
| Credential data (other credential types)   | SD-JWT /<br/> mdoc | Wallet app database    | Other issuers  | Disclosure of data to RP's and Issuers
| Wallet Certificate                         | JWT                | Wallet App Database    | Wallet Provider |  Authenticating Wallet to WP Wallet Provider
| Wallet Unit Attestation                    | SD-JWT             | Transient (passed to PID Issuer) | Wallet Provider   | Authenticating Wallet Unit to PID Issuer


### Cryptographic keys (Supporting)
| Cryptographic Key                                | Alg./ Size          | Store                  | Usage
|--------------------------------------------------|---------------------|------------------------|----------------|
| Wallet database encryption key                   |                     | ?                      | Wallet DB encryption

