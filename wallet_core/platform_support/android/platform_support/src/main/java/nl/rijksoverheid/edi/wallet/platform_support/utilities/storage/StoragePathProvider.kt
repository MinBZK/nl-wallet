package nl.rijksoverheid.edi.wallet.platform_support.utilities.storage

interface StoragePathProvider {
    fun getStoragePath(): String
}