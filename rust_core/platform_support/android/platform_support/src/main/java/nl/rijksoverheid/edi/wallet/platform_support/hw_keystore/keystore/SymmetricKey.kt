// Inspired by IRMAMobile: https://github.com/privacybydesign/irmamobile/blob/v6.4.1/android/app/src/main/java/foundation/privacybydesign/irmamobile/irma_mobile_bridge/ECDSA.java
package nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.keystore

import android.content.Context
import android.util.Log
import androidx.security.crypto.EncryptedFile
import nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.util.toByteArray
import nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.util.toUByteList
import uniffi.hw_keystore.EncryptionKeyBridge
import java.io.File
import java.security.KeyStore

private const val KEYSTORE_PROVIDER = "AndroidKeyStore"

class SymmetricKey(private val context: Context, private val keyAlias: String) :
    EncryptionKeyBridge {

    private val keyStore: KeyStore = KeyStore.getInstance(KEYSTORE_PROVIDER)

    init {
        keyStore.load(null)
    }

    override fun encrypt(payload: List<UByte>): List<UByte> {
        Log.d("SymmetricKey", "Encrypt input: ${String(payload.toByteArray())}")
        val file = File(context.filesDir, "key")
        val encryptFile = EncryptedFile.Builder(
            file,
            context,
            keyAlias,
            EncryptedFile.FileEncryptionScheme.AES256_GCM_HKDF_4KB
        ).build()
        encryptFile.openFileOutput().use {
            it.write(payload.toByteArray())
            it.flush()
        }
        return file.readBytes().toUByteList().also { file.delete() }
    }

    override fun decrypt(payload: List<UByte>): List<UByte> {
        Log.d("SymmetricKey", "Decrypt input: ${String(payload.toByteArray())}")
        val file = File(context.filesDir, "key")
        file.writeBytes(payload.toByteArray())
        Log.d("SymmetricKey", "Decrypt file contents: ${String(file.readBytes())}")
        val encryptFile = EncryptedFile.Builder(
            file,
            context,
            keyAlias,
            EncryptedFile.FileEncryptionScheme.AES256_GCM_HKDF_4KB
        ).build()
        encryptFile.openFileInput()
            .use { return it.readBytes().toUByteList().also { file.delete() } }
    }

}