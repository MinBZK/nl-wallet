package nl.rijksoverheid.edi.wallet.platform_support.utilities

import nl.rijksoverheid.edi.wallet.platform_support.PlatformSupportInitializer
import nl.rijksoverheid.edi.wallet.platform_support.utilities.storage.StoragePathProvider
import uniffi.platform_support.UtilitiesBridge as RustUtilitiesBridge

/**
 * This class is automatically initialized on app start through
 * the [PlatformSupportInitializer] class.
 */
class UtilitiesBridge(private val pathProvider: StoragePathProvider) : RustUtilitiesBridge {

    override fun getStoragePath() = pathProvider.getStoragePath()
}
