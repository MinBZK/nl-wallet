# Mdoc issuance protocol archive

This folder contains an archive of an issuance protocol for ISO mdocs that was previously used in the NL Wallet.
This protocol was based on the ISO 23220-3 standard and included an extension of the BasicSA protocol defined in that standard.
It was able to issue multiple distinct mdocs within a single session, as well as multiple copies of each of those mdocs.
During issuance, it sent to the wallet along with the nonce an unsigned preview of the mdocs that the wallet would receive.

Instead of this protocol, the NL Wallet now uses the [OpenID4VCI](https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html) protocol for issuance of mdocs.

This archive is only for demonstrative purposes and it is not in working condition.
It does not contain all code that was involved in this protocol, but instead only the most important files that were only concerned with the protocol.

The last commit in which this protocol was in a working state has a git tag on it called `mdoc-issuance-protocol`.
