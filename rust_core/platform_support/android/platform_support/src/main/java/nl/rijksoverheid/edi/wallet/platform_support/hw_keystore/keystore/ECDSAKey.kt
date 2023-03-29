// Inspired by IRMAMobile: https://github.com/privacybydesign/irmamobile/blob/v6.4.1/android/app/src/main/java/foundation/privacybydesign/irmamobile/irma_mobile_bridge/ECDSA.java
package nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.keystore

import android.os.Build
import android.security.keystore.KeyInfo
import android.security.keystore.KeyProperties
import android.util.Log
import androidx.annotation.VisibleForTesting
import nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.keystore.KeyStoreKeyError.*
import nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.util.toByteArray
import nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.util.toUByteList
import uniffi.hw_keystore.SigningKeyBridge
import java.security.KeyFactory
import java.security.KeyStore
import java.security.KeyStoreException
import java.security.NoSuchAlgorithmException
import java.security.PrivateKey
import java.security.Signature
import java.security.UnrecoverableKeyException

private const val KEYSTORE_PROVIDER = "AndroidKeyStore"

@VisibleForTesting
const val SIGNATURE_ALGORITHM = "SHA256withECDSA"

class ECDSAKey(private val keyAlias: String) : SigningKeyBridge {
    private val keyStore: KeyStore = KeyStore.getInstance(KEYSTORE_PROVIDER)

    init {
        keyStore.load(null)
    }

    @Throws(uniffi.hw_keystore.KeyStoreException.KeyException::class)
    override fun publicKey(): List<UByte> {
        try {
            return keyStore.getCertificate(keyAlias).publicKey.encoded.toUByteList()
        } catch (ex: Exception) {
            throw DeriveKeyError(ex).keyException
        }
    }

    @Throws(uniffi.hw_keystore.KeyStoreException.KeyException::class)
    override fun sign(payload: List<UByte>): List<UByte> {
        try {
            val signature = Signature.getInstance(SIGNATURE_ALGORITHM)
            val privateKey = keyStore.getKey(keyAlias, null) as PrivateKey
            signature.initSign(privateKey)
            signature.update(payload.toByteArray())
            return signature.sign().toUByteList()
        } catch (ex: Exception) {
            when (ex) {
                is UnrecoverableKeyException,
                is NoSuchAlgorithmException,
                is KeyStoreException -> throw FetchKeyError(ex).keyException
            }
            throw SignKeyError(ex).keyException
        }
    }

    val isHardwareBacked: Boolean
        get() {
            try {
                val keyInfo = this.keyInfo
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                    if (keyInfo.securityLevel == KeyProperties.SECURITY_LEVEL_TRUSTED_ENVIRONMENT) return true
                    if (keyInfo.securityLevel == KeyProperties.SECURITY_LEVEL_STRONGBOX) return true
                    return false
                } else {
                    @Suppress("DEPRECATION")
                    return keyInfo.isInsideSecureHardware
                }
            } catch (e: Exception) {
                Log.e("ECDSAKey", Log.getStackTraceString(e))
                return false
            }
        }

    /**
     * Returns the securityLevel of this key, falls back to providing
     * null on devices with API level < 31.
     */
    val securityLevelCompat: Int?
        get() = runCatching<Int> {
            return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                keyInfo.securityLevel
            } else {
                null
            }
        }.getOrNull()

    private val keyInfo: KeyInfo
        get() {
            val privateKey = keyStore.getKey(keyAlias, null)
            val keyFactory: KeyFactory =
                KeyFactory.getInstance(privateKey.algorithm, KEYSTORE_PROVIDER)
            return keyFactory.getKeySpec(privateKey, KeyInfo::class.java)
        }
}