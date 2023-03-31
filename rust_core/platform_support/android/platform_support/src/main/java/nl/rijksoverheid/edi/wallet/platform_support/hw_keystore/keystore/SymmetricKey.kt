// Inspired by AndroidCrypto: https://github.com/philipplackner/AndroidCrypto/issues/2#issuecomment-1267021656
package nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.keystore

import android.os.Build
import android.security.keystore.KeyInfo
import android.security.keystore.KeyProperties
import android.util.Log
import nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.util.toByteArray
import nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.util.toUByteList
import uniffi.platform_support.EncryptionKeyBridge
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import java.security.KeyFactory
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.SecretKey
import javax.crypto.spec.IvParameterSpec

private const val KEYSTORE_PROVIDER = "AndroidKeyStore"

class SymmetricKey(private val keyAlias: String) :
    EncryptionKeyBridge {

    private val keyStore: KeyStore = KeyStore.getInstance(KEYSTORE_PROVIDER)

    companion object {
        const val ALGORITHM = KeyProperties.KEY_ALGORITHM_AES
        const val CHUNK_SIZE = 1024 // bytes
        const val KEY_SIZE = 16 // bytes
        const val BLOCK_MODE = KeyProperties.BLOCK_MODE_CBC
        const val PADDING = KeyProperties.ENCRYPTION_PADDING_PKCS7
        const val TRANSFORMATION = "$ALGORITHM/$BLOCK_MODE/$PADDING"
    }

    init {
        keyStore.load(null)
    }

    override fun encrypt(payload: List<UByte>): List<UByte> {
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

    override fun decrypt(payload: List<UByte>): List<UByte> {
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

    private val encryptCipher
        get() = Cipher.getInstance(TRANSFORMATION).apply {
            init(Cipher.ENCRYPT_MODE, getKey())
        }

    private fun getDecryptCipherForIv(initVector: ByteArray): Cipher {
        return Cipher.getInstance(TRANSFORMATION).apply {
            val ivSpec = IvParameterSpec(initVector)
            init(Cipher.DECRYPT_MODE, getKey(), ivSpec)
        }
    }

    private fun getKey(): SecretKey =
        (keyStore.getEntry(keyAlias, null) as KeyStore.SecretKeyEntry).secretKey

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
                Log.e("SymmetricKey", Log.getStackTraceString(e))
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
