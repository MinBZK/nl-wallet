package nl.rijksoverheid.edi.wallet.platform_support.utilities.storage

import android.content.Context

class StoragePathProviderImpl(private val context: Context) : StoragePathProvider {

    //The returned path may change over time if the calling app is moved to an
    //adopted storage device, so only relative paths should be persisted.
    override fun getStoragePath(): String = context.filesDir.absolutePath
}