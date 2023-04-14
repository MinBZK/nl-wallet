package nl.rijksoverheid.edi.wallet.platform_support.utilities

import androidx.annotation.VisibleForTesting
import nl.rijksoverheid.edi.wallet.platform_support.PlatformSupportInitializer
import nl.rijksoverheid.edi.wallet.platform_support.utilities.storage.StoragePathProvider
import uniffi.platform_support.UtilitiesBridge

/**
 * This class is automatically initialized on app start through
 * the [PlatformSupportInitializer] class.
 */
class NativeUtilitiesBridge(private val pathProvider: StoragePathProvider) : UtilitiesBridge {

    companion object {
        @VisibleForTesting
        lateinit var bridge: NativeUtilitiesBridge
    }

    init {
        bridge = this
    }

    override fun getStoragePath() = pathProvider.getStoragePath()
}
