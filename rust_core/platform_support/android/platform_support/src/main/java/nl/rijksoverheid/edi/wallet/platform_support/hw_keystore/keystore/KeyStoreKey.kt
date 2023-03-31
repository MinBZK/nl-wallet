package nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.keystore

import android.os.Build
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