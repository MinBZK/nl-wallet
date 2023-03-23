package nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.ecdsa

import uniffi.hw_keystore.KeyStoreException.KeyException

enum class ECDSAErrors {
    DERIVE, SIGN, CREATE, FETCH, MISSING_HARDWARE;

    fun asKeyException(): KeyException {
        return when (this) {
            DERIVE -> KeyException("Could not derive public key")
            SIGN -> KeyException("Could not sign with private key")
            CREATE -> KeyException("Could not create private key")
            FETCH -> KeyException("Could not fetch private key")
            MISSING_HARDWARE -> KeyException("Could not generate hardware backed key")
        }
    }
}