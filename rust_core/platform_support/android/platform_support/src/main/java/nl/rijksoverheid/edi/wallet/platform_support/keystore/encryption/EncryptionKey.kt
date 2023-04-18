// Inspired by AndroidCrypto: https://github.com/philipplackner/AndroidCrypto/issues/2#issuecomment-1267021656
package nl.rijksoverheid.edi.wallet.platform_support.keystore.encryption

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyInfo
import android.security.keystore.KeyProperties
import nl.rijksoverheid.edi.wallet.platform_support.keystore.KEYSTORE_PROVIDER
import nl.rijksoverheid.edi.wallet.platform_support.keystore.KeyStoreKey
import nl.rijksoverheid.edi.wallet.platform_support.keystore.setStrongBoxBackedCompat
import nl.rijksoverheid.edi.wallet.platform_support.util.toByteArray
import nl.rijksoverheid.edi.wallet.platform_support.util.toUByteList
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import java.security.KeyStore
import java.security.NoSuchAlgorithmException
import java.security.NoSuchProviderException
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.SecretKeyFactory
import javax.crypto.spec.IvParameterSpec

class EncryptionKey(keyAlias: String) : KeyStoreKey(keyAlias) {

    companion object {
        private const val ALGORITHM = KeyProperties.KEY_ALGORITHM_AES
        private const val CHUNK_SIZE = 1024 // bytes
        private const val KEY_SIZE = 16 // bytes
        private const val BLOCK_MODE = KeyProperties.BLOCK_MODE_CBC
        private const val PADDING = KeyProperties.ENCRYPTION_PADDING_PKCS7
        private const val TRANSFORMATION = "$ALGORITHM/$BLOCK_MODE/$PADDING"

        @Throws(
            NoSuchProviderException::class,
            NoSuchAlgorithmException::class,
            IllegalStateException::class
        )
        fun createKey(context: Context, alias: String) {
            val spec = KeyGenParameterSpec.Builder(
                alias,
                KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT
            ).setBlockModes(BLOCK_MODE)
                .setEncryptionPaddings(PADDING)
                .setKeySize(KEY_SIZE * 8 /* in bits */)
                .setUserAuthenticationRequired(false)
                .setRandomizedEncryptionRequired(true)
                .setStrongBoxBackedCompat(context, true)

            KeyGenerator.getInstance(ALGORITHM, KEYSTORE_PROVIDER).apply {
                init(spec.build())
                generateKey()
            }
        }
    }

    fun encrypt(payload: List<UByte>): List<UByte> {
        val cipher = encryptCipher
        val bytes = payload.toByteArray()
        val iv = cipher.iv
        val outputStream = ByteArrayOutputStream()
        outputStream.use {
            it.write(iv)
            // write the payload in chunks to make sure to support larger data amounts (this would otherwise fail silently and result in corrupted data being read back)
            ////////////////////////////////////
            val inputStream = ByteArrayInputStream(bytes)
            val buffer = ByteArray(CHUNK_SIZE)
            while (inputStream.available() > CHUNK_SIZE) {
                inputStream.read(buffer)
                val ciphertextChunk = cipher.update(buffer)
                it.write(ciphertextChunk)
            }
            // the last chunk must be written using doFinal() because this takes the padding into account
            val remainingBytes = inputStream.readBytes()
            val lastChunk = cipher.doFinal(remainingBytes)
            it.write(lastChunk)
            //////////////////////////////////
        }
        return outputStream.toByteArray().toUByteList()

    }

    fun decrypt(payload: List<UByte>): List<UByte> {
        val inputStream = ByteArrayInputStream(payload.toByteArray())
        return inputStream.use {
            val iv = ByteArray(KEY_SIZE)
            it.read(iv)
            val cipher = getDecryptCipherForIv(iv)
            val outputStream = ByteArrayOutputStream()

            // read the payload in chunks to make sure to support larger data amounts (this would otherwise fail silently and result in corrupted data being read back)
            ////////////////////////////////////
            val buffer = ByteArray(CHUNK_SIZE)
            while (inputStream.available() > CHUNK_SIZE) {
                inputStream.read(buffer)
                val ciphertextChunk = cipher.update(buffer)
                outputStream.write(ciphertextChunk)
            }
            // the last chunk must be read using doFinal() because this takes the padding into account
            val remainingBytes = inputStream.readBytes()
            val lastChunk = cipher.doFinal(remainingBytes)
            outputStream.write(lastChunk)
            //////////////////////////////////

            outputStream.toByteArray().toUByteList()
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

    private val encryptCipher
        get() = Cipher.getInstance(TRANSFORMATION).apply {
            init(Cipher.ENCRYPT_MODE, secretKey)
        }

    private fun getDecryptCipherForIv(initVector: ByteArray): Cipher {
        return Cipher.getInstance(TRANSFORMATION).apply {
            val ivSpec = IvParameterSpec(initVector)
            init(Cipher.DECRYPT_MODE, secretKey, ivSpec)
        }
    }
}
