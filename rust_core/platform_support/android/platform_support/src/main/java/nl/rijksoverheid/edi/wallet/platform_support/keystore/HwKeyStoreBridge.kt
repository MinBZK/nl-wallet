// Inspired by IRMAMobile: https://github.com/privacybydesign/irmamobile/blob/v6.4.1/android/app/src/main/java/foundation/privacybydesign/irmamobile/irma_mobile_bridge/ECDSA.java
package nl.rijksoverheid.edi.wallet.platform_support.keystore

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import androidx.annotation.VisibleForTesting
import nl.rijksoverheid.edi.wallet.platform_support.BuildConfig
import nl.rijksoverheid.edi.wallet.platform_support.PlatformSupportInitializer
import nl.rijksoverheid.edi.wallet.platform_support.util.DeviceUtils.isRunningOnEmulator
import nl.rijksoverheid.edi.wallet.platform_support.util.isDeviceLocked
import uniffi.platform_support.EncryptionKeyBridge
import uniffi.platform_support.KeyStoreBridge
import uniffi.platform_support.SigningKeyBridge
import java.security.KeyStore
import java.security.KeyStoreException

const val KEYSTORE_PROVIDER = "AndroidKeyStore"
private const val SIGN_KEY_PREFIX = "ecdsa_"
private const val ENCRYPT_KEY_PREFIX = "aes_"

/**
 * This class is automatically initialized on app start through
 * the [PlatformSupportInitializer] class.
 */
class HwKeyStoreBridge(private val context: Context) : KeyStoreBridge {
    companion object {
        @VisibleForTesting
        lateinit var bridge: KeyStoreBridge
    }

    private val keyStore: KeyStore = KeyStore.getInstance(KEYSTORE_PROVIDER)

    init {
        keyStore.load(null)
        bridge = this
    }

    @Throws(uniffi.platform_support.KeyStoreException::class)
    override fun getOrCreateSigningKey(identifier: String): SigningKeyBridge {
        val alias = SIGN_KEY_PREFIX + identifier
        try {
            verifyDeviceUnlocked()
            if (!keyExists(alias)) ECDSAKey.createKey(context, alias)
            return ECDSAKey(alias).takeIf { it.isConsideredValid }!!
        } catch (ex: Exception) {
            if (ex is uniffi.platform_support.KeyStoreException) throw ex
            throw KeyStoreKeyError.CreateKeyError(ex).keyException
        }
    }

    override fun getOrCreateEncryptionKey(identifier: String): EncryptionKeyBridge {
        val alias = ENCRYPT_KEY_PREFIX + identifier;
        try {
            verifyDeviceUnlocked()
            if (!keyExists(alias)) AESKey.createKey(context, alias)
            return AESKey(alias).takeIf { it.isConsideredValid }!!
        } catch (ex: Exception) {
            if (ex is uniffi.platform_support.KeyStoreException) throw ex
            throw KeyStoreKeyError.CreateKeyError(ex).keyException
        }
    }

    @VisibleForTesting
    fun clean() = keyStore.aliases().asSequence().forEach(::deleteEntry)

    private fun deleteEntry(identifier: String) = keyStore.deleteEntry(identifier)

    @Throws(KeyStoreException::class)
    private fun keyExists(keyAlias: String): Boolean = keyStore.containsAlias(keyAlias)

    /**
     * Verifies that the device currently unlocked. Something we require
     * before creating or fetching a key.
     *
     * Note: Ideally we configure the [KeyGenParameterSpec.Builder]
     * with setUnlockedDeviceRequired(true), but this is throws in some
     * cases, see: Issue tracker: https://issuetracker.google.com/u/1/issues/191391068
     * As such, validating it manually.
     */
    @Throws(IllegalStateException::class)
    private fun verifyDeviceUnlocked() {
        if (context.isDeviceLocked()) {
            throw IllegalStateException("Key interaction not allowed while device is locked")
        }
    }
}

private val KeyStoreKey.isConsideredValid: Boolean
    @Throws(uniffi.platform_support.KeyStoreException.KeyException::class)
    get() {
        val allowSoftwareBackedKeys = isRunningOnEmulator && BuildConfig.DEBUG
        return when {
            isHardwareBacked -> true
            allowSoftwareBackedKeys -> true
            !isHardwareBacked && !allowSoftwareBackedKeys -> {
                throw KeyStoreKeyError.MissingHardwareError(securityLevelCompat).keyException
            }
            else -> false
        }
    }