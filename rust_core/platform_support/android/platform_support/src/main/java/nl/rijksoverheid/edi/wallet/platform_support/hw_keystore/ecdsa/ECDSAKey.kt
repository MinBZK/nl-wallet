// Inspired by IRMAMobile: https://github.com/privacybydesign/irmamobile/blob/v6.4.1/android/app/src/main/java/foundation/privacybydesign/irmamobile/irma_mobile_bridge/ECDSA.java
package nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.ecdsa

import android.os.Build
import android.security.keystore.KeyInfo
import android.security.keystore.KeyProperties
import android.util.Log
import androidx.annotation.VisibleForTesting
import nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.ecdsa.KeyStoreKeyError.*
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

private const val keyStoreProvider = "AndroidKeyStore"

@VisibleForTesting
const val signatureAlgorithm = "SHA256withECDSA"

class ECDSAKey(private val keyAlias: String) : SigningKeyBridge {
    private val keyStore: KeyStore = KeyStore.getInstance(keyStoreProvider)

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
            val signature = Signature.getInstance(signatureAlgorithm)
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
                val privateKey = keyStore.getKey(keyAlias, null)
                val keyFactory: KeyFactory =
                    KeyFactory.getInstance(privateKey.algorithm, keyStoreProvider)
                val keyInfo: KeyInfo = keyFactory.getKeySpec(privateKey, KeyInfo::class.java)
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

}