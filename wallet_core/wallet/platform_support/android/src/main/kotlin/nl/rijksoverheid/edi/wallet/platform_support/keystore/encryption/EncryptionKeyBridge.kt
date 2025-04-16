package nl.rijksoverheid.edi.wallet.platform_support.keystore.encryption

import android.content.Context
import nl.rijksoverheid.edi.wallet.platform_support.keystore.KeyBridge
import nl.rijksoverheid.edi.wallet.platform_support.keystore.KeyStoreKeyError
import uniffi.platform_support.KeyStoreException
import uniffi.platform_support.EncryptionKeyBridge as RustEncryptionBridge

private const val ENCRYPT_KEY_PREFIX = "aes_"

class EncryptionKeyBridge(context: Context) : KeyBridge(context), RustEncryptionBridge {

    @Throws(KeyStoreException::class)
    fun getOrCreateKey(identifier: String): EncryptionKey {
        val keyAlias = ENCRYPT_KEY_PREFIX + identifier
        try {
            verifyDeviceUnlocked()
            if (!keyExists(keyAlias)) EncryptionKey.createKey(context, keyAlias)
            return EncryptionKey(keyAlias)
        } catch (ex: Exception) {
            if (ex is KeyStoreException) throw ex
            throw KeyStoreKeyError.CreateKeyError(ex).keyException
        }
    }

    override fun encrypt(identifier: String, payload: List<UByte>): List<UByte> {
        val key = getOrCreateKey(identifier)
        return key.encrypt(payload)
    }

    override fun decrypt(identifier: String, payload: List<UByte>): List<UByte> {
        val key = getOrCreateKey(identifier)
        return key.decrypt(payload)
    }

    override fun delete(identifier: String) {
        val keyAlias = ENCRYPT_KEY_PREFIX + identifier
        keyStore.deleteEntry(keyAlias)
    }

    override fun clean() =
        keyStore.aliases().asSequence().filter { it.startsWith(ENCRYPT_KEY_PREFIX) }
            .forEach(::deleteEntry)
}
