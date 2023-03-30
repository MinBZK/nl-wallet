package nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.utilities

import android.content.Context
import nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.PlatformSupportInitializer
import uniffi.platform_support.UtilitiesBridge
import uniffi.platform_support.initUtilities

/**
 * This class is automatically initialized on app start through
 * the [PlatformSupportInitializer] class.
 */
class NativeUtilitiesBridge(private val context: Context) : UtilitiesBridge {

    init {
        initUtilities(this)
    }

    override fun getStoragePath(): String {
        //The returned path may change over time if the calling app is moved to an
        //adopted storage device, so only relative paths should be persisted.
        return context.filesDir.absolutePath
    }
}
