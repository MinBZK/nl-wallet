// Inspired by IRMAMobile: https://github.com/privacybydesign/irmamobile/blob/v6.4.1/android/app/src/main/java/foundation/privacybydesign/irmamobile/irma_mobile_bridge/ECDSA.java
package nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.keystore

import android.content.Context
import androidx.annotation.VisibleForTesting
import nl.rijksoverheid.edi.wallet.platform_support.BuildConfig
import nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.PlatformSupportInitializer
import nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.util.DeviceUtils.isRunningOnEmulator
import uniffi.platform_support.EncryptionKeyBridge
import uniffi.platform_support.KeyStoreBridge
import uniffi.platform_support.SigningKeyBridge
import uniffi.platform_support.initHwKeystore
import java.security.KeyStore
import java.security.KeyStoreException

const val KEYSTORE_PROVIDER = "AndroidKeyStore"

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
        initHwKeystore(this)
    }

    @Throws(uniffi.platform_support.KeyStoreException::class)
    override fun getOrCreateSigningKey(identifier: String): SigningKeyBridge {
        val alias = "ecdsa_$identifier"
        try {
            if (!keyExists(alias)) ECDSAKey.createKey(context, alias)
            return ECDSAKey(alias).takeIf { it.isConsideredValid }!!
        } catch (ex: Exception) {
            if (ex is uniffi.platform_support.KeyStoreException) throw ex
            throw KeyStoreKeyError.CreateKeyError(ex).keyException
        }
    }

    override fun getOrCreateEncryptionKey(identifier: String): EncryptionKeyBridge {
        val alias = "aes_$identifier"
        try {
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
}

private val KeyStoreKey.isConsideredValid: Boolean
    @Throws(uniffi.platform_support.KeyStoreException.KeyException::class)
    get() {
        val allowSoftwareBackedKeys = isRunningOnEmulator && BuildConfig.DEBUG
        return when {
            this.isHardwareBacked -> true
            allowSoftwareBackedKeys -> true
            else -> throw KeyStoreKeyError.MissingHardwareError(this.securityLevelCompat).keyException
        }
    }