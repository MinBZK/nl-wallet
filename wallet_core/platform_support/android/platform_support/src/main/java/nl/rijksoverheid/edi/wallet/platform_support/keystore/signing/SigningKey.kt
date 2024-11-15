// Inspired by IRMAMobile: https://github.com/privacybydesign/irmamobile/blob/v6.4.1/android/app/src/main/java/foundation/privacybydesign/irmamobile/irma_mobile_bridge/ECDSA.java
package nl.rijksoverheid.edi.wallet.platform_support.keystore.signing

import android.content.Context
import android.os.Build
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyInfo
import android.security.keystore.KeyProperties
import androidx.annotation.VisibleForTesting
import nl.rijksoverheid.edi.wallet.platform_support.keystore.KEYSTORE_PROVIDER
import nl.rijksoverheid.edi.wallet.platform_support.keystore.KeyStoreKey
import nl.rijksoverheid.edi.wallet.platform_support.keystore.KeyStoreKeyError
import nl.rijksoverheid.edi.wallet.platform_support.keystore.setStrongBoxBackedCompat
import nl.rijksoverheid.edi.wallet.platform_support.util.toByteArray
import nl.rijksoverheid.edi.wallet.platform_support.util.toUByteList
import uniffi.platform_support.KeyStoreException.*
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

class SigningKey(keyAlias: String) : KeyStoreKey(keyAlias) {

    companion object {
        @Throws(
            NoSuchProviderException::class,
            NoSuchAlgorithmException::class,
            IllegalStateException::class
        )
        fun createKey(context: Context, keyAlias: String, challenge: List<UByte>? = null) {
            val spec = KeyGenParameterSpec.Builder(keyAlias, KeyProperties.PURPOSE_SIGN)
                .setAlgorithmParameterSpec(ECGenParameterSpec("secp256r1"))
                .setDigests(KeyProperties.DIGEST_SHA256)
                .setStrongBoxBackedCompat(context, true)
                .also { spec ->
                    challenge?.let {
                        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
                            spec.setAttestationChallenge(it.toByteArray())
                        }
                    }
                }

            KeyPairGenerator.getInstance(
                KeyProperties.KEY_ALGORITHM_EC,
                KEYSTORE_PROVIDER
            ).apply {
                initialize(spec.build())
                generateKeyPair()
            }
        }
    }

    @Throws(KeyException::class)
    fun publicKey(): List<UByte> {
        try {
            return keyStore.getCertificate(keyAlias).publicKey.encoded.toUByteList()
        } catch (ex: Exception) {
            throw KeyStoreKeyError.DeriveKeyError(ex).keyException
        }
    }

    @Throws(KeyException::class)
    fun sign(payload: List<UByte>): List<UByte> {
        try {
            val privateKey = keyStore.getKey(keyAlias, null) as PrivateKey
            return Signature.getInstance(SIGNATURE_ALGORITHM).run {
                initSign(privateKey)
                update(payload.toByteArray())
                sign().toUByteList()
            }
        } catch (ex: Exception) {
            when (ex) {
                is UnrecoverableKeyException,
                is NoSuchAlgorithmException,
                is KeyStoreException -> throw KeyStoreKeyError.FetchKeyError(ex).keyException
            }
            throw KeyStoreKeyError.SignKeyError(ex).keyException
        }
    }

    override val keyInfo: KeyInfo
        get() {
            val privateKey = keyStore.getKey(keyAlias, null)
            val keyFactory: KeyFactory =
                KeyFactory.getInstance(privateKey.algorithm, KEYSTORE_PROVIDER)
            return keyFactory.getKeySpec(privateKey, KeyInfo::class.java)
        }
}
