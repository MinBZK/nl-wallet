package nl.rijksoverheid.edi.wallet.platform_support.keystore.encryption

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyInfo
import android.security.keystore.KeyProperties
import nl.rijksoverheid.edi.wallet.platform_support.keystore.KEYSTORE_PROVIDER
import nl.rijksoverheid.edi.wallet.platform_support.keystore.KeyStoreKey
import nl.rijksoverheid.edi.wallet.platform_support.keystore.setStrongBoxBackedCompat
import nl.rijksoverheid.edi.wallet.platform_support.utilities.toByteArray
import nl.rijksoverheid.edi.wallet.platform_support.utilities.toUByteList
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import java.security.KeyStore
import java.security.NoSuchAlgorithmException
import java.security.NoSuchProviderException
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.SecretKeyFactory
import javax.crypto.spec.GCMParameterSpec

class EncryptionKey(keyAlias: String) : KeyStoreKey(keyAlias) {

    companion object {
        private const val ALGORITHM = KeyProperties.KEY_ALGORITHM_AES
        private const val BLOCK_MODE = KeyProperties.BLOCK_MODE_GCM
        private const val PADDING = KeyProperties.ENCRYPTION_PADDING_NONE
        private const val TRANSFORMATION = "$ALGORITHM/$BLOCK_MODE/$PADDING"
        private const val KEY_SIZE_BITS = 256
        private const val AUTH_TAG_SIZE_BITS = 128
        private const val IV_SIZE = 12 // bytes

        @Throws(
            NoSuchProviderException::class,
            NoSuchAlgorithmException::class,
            IllegalStateException::class
        )
        fun createKey(context: Context, keyAlias: String) {
            val spec = KeyGenParameterSpec.Builder(
                keyAlias,
                KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT
            ).setKeySize(KEY_SIZE_BITS)
                .setBlockModes(BLOCK_MODE)
                .setEncryptionPaddings(PADDING)
                .setUserAuthenticationRequired(false)
                .setRandomizedEncryptionRequired(true)
                .setStrongBoxBackedCompat(context, true)
                .build()


            KeyGenerator.getInstance(ALGORITHM, KEYSTORE_PROVIDER).run {
                init(spec)
                generateKey()
            }
        }
    }

    fun encrypt(payload: List<UByte>): List<UByte> {
        val encryptedPayload = encryptCipher.doFinal(payload.toByteArray())
        val initVector = encryptCipher.iv

        assert(initVector.size == IV_SIZE, { "Unexpected IV size. Found: ${initVector.size}, expected: $IV_SIZE" })

        val outputStream = ByteArrayOutputStream()
        outputStream.use {
            it.write(initVector)
            it.write(encryptedPayload)
        }

        return outputStream.toByteArray().toUByteList()
    }

    fun decrypt(payload: List<UByte>): List<UByte> {
        val inputStream = ByteArrayInputStream(payload.toByteArray())
        inputStream.use {
            val initVector = ByteArray(IV_SIZE)
            it.read(initVector, 0, IV_SIZE)
            val cipher = getDecryptCipherForIv(initVector)

            val encryptedBytes = ByteArray(payload.size - IV_SIZE)
            it.read(encryptedBytes)

            val decrypted = cipher.doFinal(encryptedBytes)
            return decrypted.toUByteList()
        }
    }

    override val keyInfo: KeyInfo
        get() {
            val secretKeyFactory: SecretKeyFactory =
                SecretKeyFactory.getInstance(secretKey.algorithm, KEYSTORE_PROVIDER)
            return secretKeyFactory.getKeySpec(
                secretKey,
                KeyInfo::class.java
            ) as KeyInfo
        }

    private val secretKey: SecretKey
        get() = (keyStore.getEntry(keyAlias, null) as KeyStore.SecretKeyEntry).secretKey

    private val encryptCipher: Cipher by lazy {
        Cipher.getInstance(TRANSFORMATION).apply {
            init(Cipher.ENCRYPT_MODE, secretKey)
        }
    }

    private fun getDecryptCipherForIv(initVector: ByteArray): Cipher {
        val gcmParameterSpec = GCMParameterSpec(AUTH_TAG_SIZE_BITS, initVector)
        return Cipher.getInstance(TRANSFORMATION).apply {
            init(Cipher.DECRYPT_MODE, secretKey, gcmParameterSpec)
        }
    }
}
