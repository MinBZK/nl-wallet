package nl.rijksoverheid.edi.wallet.platform_support.hw_keystore

import nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.bridge.PlatformKeyStore
import uniffi.hw_keystore.KeyStoreBridge
import uniffi.hw_keystore.initHwKeystore

class HWKeyStore {
    companion object {
        val shared = HWKeyStore()

        private val keyStore: KeyStoreBridge = PlatformKeyStore()

        init {
            initHwKeystore(bridge = keyStore)
        }
    }
}
