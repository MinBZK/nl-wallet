package nl.rijksoverheid.edi.wallet.platform_support.keystore

import android.content.Context
import androidx.annotation.VisibleForTesting
import nl.rijksoverheid.edi.wallet.platform_support.utilities.isDeviceLocked
import uniffi.platform_support.KeyStoreException
import java.security.KeyStore

const val KEYSTORE_PROVIDER = "AndroidKeyStore"

abstract class KeyBridge(val context: Context) {

    protected val keyStore: KeyStore = KeyStore.getInstance(KEYSTORE_PROVIDER)

    init {
        keyStore.load(null)
    }

    /**
     * Verifies that the device is currently unlocked. Something we require
     * before creating or fetching a key.
     *
     * Note: Ideally we configure the [KeyGenParameterSpec.Builder]
     * with setUnlockedDeviceRequired(true), but this is throws in some
     * cases, see: Issue tracker: https://issuetracker.google.com/u/1/issues/191391068
     * As such, validating it manually.
     */
    @Throws(IllegalStateException::class)
    fun verifyDeviceUnlocked() {
        check(!context.isDeviceLocked()) { "Key interaction not allowed while device is locked" }
    }

    /**
     * Verifies that the keystore does not contain a key with [keyAlias].
     */
    @Throws(IllegalStateException::class)
    fun verifyKeyDoesNotExist(keyAlias: String) {
        check(!keyExists(keyAlias)) { "A key already exists with alias: `$keyAlias`" }
     }

    /**
     * Verifies that the keystore does contain a key with [keyAlias].
     */
    @Throws(IllegalStateException::class)
    fun verifyKeyExists(keyAlias: String) {
        check(keyExists(keyAlias)) { "Key not found for alias: `$keyAlias`" }
    }

    @Throws(KeyStoreException::class)
    fun keyExists(keyAlias: String): Boolean = keyStore.containsAlias(keyAlias)

    /**
     * Removes all keyAliases associated with this KeyBridge
     * from the KeyStore.
     */
    @VisibleForTesting
    abstract fun clean()

    /**
     * Deletes the key associated with the provided [keyAlias]
     * from the KeyStore.
     */
    protected fun deleteEntry(keyAlias: String) = keyStore.deleteEntry(keyAlias)

}

