package nl.rijksoverheid.edi.wallet.platform_support.hw_keystore

import android.content.Context
import androidx.annotation.VisibleForTesting
import uniffi.hw_keystore.StorageBridge
import uniffi.hw_keystore.initStorage

/**
 * This class is automatically initialized on app start through
 * the [PlatformSupportInitializer] class.
 */
class Storage(context: Context) {

    init {
        bridge = AndroidStorageBridge(context)
        initStorage(bridge)
    }

    companion object {
        @VisibleForTesting
        lateinit var bridge: StorageBridge
    }
}

class AndroidStorageBridge(private val context: Context) : StorageBridge {
    override fun getStoragePath(): String {
        //The returned path may change over time if the calling app is moved to an
        //adopted storage device, so only relative paths should be persisted.
        return context.filesDir.absolutePath
    }

}