package nl.rijksoverheid.edi.wallet.hwkeystore.bridge

import uniffi.hw_keystore.KeyStoreBridge
import uniffi.hw_keystore.SigningKeyBridge

class PlatformKeyStore : KeyStoreBridge {
    override fun getOrCreateKey(identifier: String): SigningKeyBridge {
        return SigningKey()
    }
}
