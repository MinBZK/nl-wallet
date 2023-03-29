package nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.storage

import android.content.Context
import nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.PlatformSupportInitializer
import uniffi.hw_keystore.StorageBridge
import uniffi.hw_keystore.initStorage

/**
 * This class is automatically initialized on app start through
 * the [PlatformSupportInitializer] class.
 */
class NativeStorageBridge(private val context: Context) : StorageBridge {

    init {
        initStorage(this)
    }

    override fun getStoragePath(): String {
        //The returned path may change over time if the calling app is moved to an
        //adopted storage device, so only relative paths should be persisted.
        return context.filesDir.absolutePath
    }
}