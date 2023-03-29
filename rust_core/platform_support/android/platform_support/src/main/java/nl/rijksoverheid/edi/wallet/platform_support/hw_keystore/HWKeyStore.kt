package nl.rijksoverheid.edi.wallet.platform_support.hw_keystore

import android.content.Context
import androidx.annotation.VisibleForTesting
import nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.ecdsa.ECDSAKeyStore
import uniffi.hw_keystore.KeyStoreBridge
import uniffi.hw_keystore.initHwKeystore

/**
 * This class is automatically initialized on app start through
 * the [PlatformSupportInitializer] class.
 */
class HWKeyStore(context: Context) {

    init {
        bridge = ECDSAKeyStore(context)
        initHwKeystore(bridge)
    }

    companion object {
        @VisibleForTesting
        lateinit var bridge: KeyStoreBridge
    }

}