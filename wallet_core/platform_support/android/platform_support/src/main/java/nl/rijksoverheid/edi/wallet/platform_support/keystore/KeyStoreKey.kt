package nl.rijksoverheid.edi.wallet.platform_support.keystore

import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyInfo
import android.security.keystore.KeyProperties
import androidx.annotation.VisibleForTesting
import nl.rijksoverheid.edi.wallet.platform_support.BuildConfig
import nl.rijksoverheid.edi.wallet.platform_support.util.DeviceUtils.isRunningOnEmulator
import java.security.KeyStore
import java.security.KeyStoreException
import java.security.cert.Certificate

abstract class KeyStoreKey(val keyAlias: String) {

    protected val keyStore: KeyStore = KeyStore.getInstance(KEYSTORE_PROVIDER)

    init {
        keyStore.load(null)
        val keyExists = keyStore.getKey(keyAlias, null) != null
        if (!keyExists) {
            throw IllegalArgumentException("No key found for $keyAlias, make sure it's created first before wrapping it in  ${this.javaClass.simpleName}")
        }
    }

    abstract val keyInfo: KeyInfo

    private val isHardwareBacked: Boolean
        get() {
            if (securityLevelCompat == KeyProperties.SECURITY_LEVEL_STRONGBOX) return true
            if (securityLevelCompat == KeyProperties.SECURITY_LEVEL_TRUSTED_ENVIRONMENT) return true
            @Suppress("DEPRECATION")
            return keyInfo.isInsideSecureHardware
        }

    /**
     * Returns the securityLevel of this key, falls back to providing
     * null on devices with API level < 31
     */
    private val securityLevelCompat: Int?
        get() {
            return runCatching<Int> {
                return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                    keyInfo.securityLevel
                } else {
                    null
                }
            }.getOrNull()
        }

    /**
     * Returns the certificate chain of this key.
     */
    @Throws(KeyStoreException::class)
    fun getCertificateChain(): Array<out Certificate>? = keyStore.getCertificateChain(keyAlias)

    val isConsideredValid: Boolean
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

    @VisibleForTesting
    fun delete() = keyStore.deleteEntry(keyAlias)
}

/**
 * Enable strongbox when this feature is available, otherwise
 * this call is simply ignored.
 */
fun KeyGenParameterSpec.Builder.setStrongBoxBackedCompat(
    context: Context,
    enable: Boolean
): KeyGenParameterSpec.Builder {
    val pm = context.packageManager
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P && pm.hasSystemFeature(PackageManager.FEATURE_STRONGBOX_KEYSTORE)) {
        this.setIsStrongBoxBacked(enable)
    }
    return this
}
