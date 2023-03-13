package nl.rijksoverheid.edi.wallet.hwkeystore

import nl.rijksoverheid.edi.wallet.hwkeystore.bridge.PlatformKeyStore
import uniffi.hw_keystore.KeyStoreBridge
import uniffi.hw_keystore.initHwKeystore

class HWKeyStore {
    companion object {
        val shared = HWKeyStore()

        private val keyStoreBride: KeyStoreBridge = PlatformKeyStore()

        init {
            initHwKeystore(bridge = keyStoreBride)
        }
    }
}
