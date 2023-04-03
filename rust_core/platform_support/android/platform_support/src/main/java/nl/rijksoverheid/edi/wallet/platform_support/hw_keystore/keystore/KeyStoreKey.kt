package nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.keystore

import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyInfo
import android.security.keystore.KeyProperties
import java.security.KeyStore

abstract class KeyStoreKey(private val keyAlias: String) {

    protected val keyStore: KeyStore = KeyStore.getInstance(KEYSTORE_PROVIDER)

    init {
        keyStore.load(null)
        assert(
            keyStore.getKey(
                keyAlias,
                null
            ) != null
        ) { "Key should be created before wrapping it in ${this.javaClass.simpleName}" }
    }

    abstract val keyInfo: KeyInfo

    val isHardwareBacked: Boolean
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
    val securityLevelCompat: Int?
        get() = runCatching<Int> {
            return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                keyInfo.securityLevel
            } else {
                null
            }
        }.getOrNull()
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