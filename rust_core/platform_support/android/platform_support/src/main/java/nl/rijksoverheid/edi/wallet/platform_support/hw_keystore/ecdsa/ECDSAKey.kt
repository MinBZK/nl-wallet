// Inspired by IRMAMobile: https://github.com/privacybydesign/irmamobile/blob/v6.4.1/android/app/src/main/java/foundation/privacybydesign/irmamobile/irma_mobile_bridge/ECDSA.java
package nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.ecdsa

import android.os.Build
import android.security.keystore.KeyInfo
import android.security.keystore.KeyProperties
import android.util.Log
import uniffi.hw_keystore.SigningKeyBridge
import java.security.KeyFactory
import java.security.KeyStore
import java.security.KeyStoreException
import java.security.NoSuchAlgorithmException
import java.security.PrivateKey
import java.security.Signature
import java.security.UnrecoverableKeyException

private const val keyStoreProvider = "AndroidKeyStore"

class ECDSAKey(private val keyAlias: String) : SigningKeyBridge {
    private val keyStore: KeyStore = KeyStore.getInstance(keyStoreProvider)

    init {
        keyStore.load(null)
    }

    @Throws(uniffi.hw_keystore.KeyStoreException.KeyException::class)
    override fun publicKey(): List<UByte> {
        try {
            return keyStore.getCertificate(keyAlias).publicKey.encoded.map { it.toUByte() }
        } catch (e: Exception) {
            throw ECDSAErrors.DERIVE.asKeyException()
        }
    }

    @Throws(uniffi.hw_keystore.KeyStoreException.KeyException::class)
    override fun sign(payload: List<UByte>): List<UByte> {
        try {
            val signature = Signature.getInstance("SHA256withECDSA")
            val privateKey = keyStore.getKey(keyAlias, null) as PrivateKey
            signature.initSign(privateKey)
            signature.update(payload.map { it.toByte() }.toByteArray())
            return signature.sign().map { it.toUByte() }
        } catch (ex: Exception) {
            when (ex) {
                is UnrecoverableKeyException,
                is NoSuchAlgorithmException,
                is KeyStoreException -> throw ECDSAErrors.FETCH.asKeyException()
            }
            throw ECDSAErrors.SIGN.asKeyException()
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