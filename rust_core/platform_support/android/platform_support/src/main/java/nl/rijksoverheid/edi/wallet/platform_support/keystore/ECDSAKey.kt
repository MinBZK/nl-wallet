// Inspired by IRMAMobile: https://github.com/privacybydesign/irmamobile/blob/v6.4.1/android/app/src/main/java/foundation/privacybydesign/irmamobile/irma_mobile_bridge/ECDSA.java
package nl.rijksoverheid.edi.wallet.platform_support.keystore

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyInfo
import android.security.keystore.KeyProperties
import androidx.annotation.VisibleForTesting
import nl.rijksoverheid.edi.wallet.platform_support.keystore.KeyStoreKeyError.*
import nl.rijksoverheid.edi.wallet.platform_support.util.toByteArray
import nl.rijksoverheid.edi.wallet.platform_support.util.toUByteList
import uniffi.platform_support.SigningKeyBridge
import java.security.KeyFactory
import java.security.KeyPairGenerator
import java.security.KeyStoreException
import java.security.NoSuchAlgorithmException
import java.security.NoSuchProviderException
import java.security.PrivateKey
import java.security.Signature
import java.security.UnrecoverableKeyException
import java.security.spec.ECGenParameterSpec


@VisibleForTesting
const val SIGNATURE_ALGORITHM = "SHA256withECDSA"

class ECDSAKey(private val keyAlias: String) : KeyStoreKey(keyAlias), SigningKeyBridge {

    companion object {
        @Throws(
            NoSuchProviderException::class,
            NoSuchAlgorithmException::class,
            IllegalStateException::class
        )
        fun createKey(context: Context, alias: String) {
            val spec = KeyGenParameterSpec.Builder(alias, KeyProperties.PURPOSE_SIGN)
                .setAlgorithmParameterSpec(ECGenParameterSpec("secp256r1"))
                .setDigests(KeyProperties.DIGEST_SHA256)
                .setStrongBoxBackedCompat(context, true)

            KeyPairGenerator.getInstance(
                KeyProperties.KEY_ALGORITHM_EC,
                KEYSTORE_PROVIDER
            ).apply {
                initialize(spec.build())
                generateKeyPair()
            }
        }
    }

    override val keyInfo: KeyInfo
        get() {
            val privateKey = keyStore.getKey(keyAlias, null)
            val keyFactory: KeyFactory =
                KeyFactory.getInstance(privateKey.algorithm, KEYSTORE_PROVIDER)
            return keyFactory.getKeySpec(privateKey, KeyInfo::class.java)
        }

    @Throws(uniffi.platform_support.KeyStoreException.KeyException::class)
    override fun publicKey(): List<UByte> {
        try {
            return keyStore.getCertificate(keyAlias).publicKey.encoded.toUByteList()
        } catch (ex: Exception) {
            throw DeriveKeyError(ex).keyException
        }
    }

    @Throws(uniffi.platform_support.KeyStoreException.KeyException::class)
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
}
