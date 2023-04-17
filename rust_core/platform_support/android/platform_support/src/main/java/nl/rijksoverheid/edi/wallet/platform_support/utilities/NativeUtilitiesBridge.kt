package nl.rijksoverheid.edi.wallet.platform_support.utilities

import nl.rijksoverheid.edi.wallet.platform_support.PlatformSupportInitializer
import nl.rijksoverheid.edi.wallet.platform_support.utilities.storage.StoragePathProvider
import uniffi.platform_support.UtilitiesBridge

/**
 * This class is automatically initialized on app start through
 * the [PlatformSupportInitializer] class.
 */
class NativeUtilitiesBridge(private val pathProvider: StoragePathProvider) : UtilitiesBridge {

    override fun getStoragePath() = pathProvider.getStoragePath()
}
