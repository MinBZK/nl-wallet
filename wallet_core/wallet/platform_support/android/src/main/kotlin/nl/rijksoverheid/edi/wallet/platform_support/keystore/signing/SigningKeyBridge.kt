package nl.rijksoverheid.edi.wallet.platform_support.keystore.signing

import android.content.Context
import nl.rijksoverheid.edi.wallet.platform_support.keystore.KeyBridge
import nl.rijksoverheid.edi.wallet.platform_support.keystore.KeyStoreKeyError
import uniffi.platform_support.KeyStoreException
import uniffi.platform_support.SigningKeyBridge as RustSigningBridge

private const val SIGN_KEY_PREFIX = "ecdsa_"

class SigningKeyBridge(context: Context) : KeyBridge(context), RustSigningBridge {

    @Throws(KeyStoreException::class)
    fun getOrCreateKey(identifier: String): SigningKey {
        val keyAlias = SIGN_KEY_PREFIX + identifier
        try {
            verifyDeviceUnlocked()
            if (!keyExists(keyAlias)) SigningKey.createKey(context, keyAlias)
            return SigningKey(keyAlias)
        } catch (ex: Exception) {
            if (ex is KeyStoreException) throw ex
            throw KeyStoreKeyError.createKeyError(ex)
        }
    }

    override fun publicKey(identifier: String): List<UByte> {
        val key = getOrCreateKey(identifier)
        return key.publicKey()
    }

    override fun sign(identifier: String, payload: List<UByte>): List<UByte> {
        val key = getOrCreateKey(identifier)
        return key.sign(payload)
    }

    override fun delete(identifier: String) {
        val keyAlias = SIGN_KEY_PREFIX + identifier
        keyStore.deleteEntry(keyAlias)
    }

    override fun clean() =
        keyStore.aliases().asSequence().filter { it.startsWith(SIGN_KEY_PREFIX) }
            .forEach(::deleteEntry)
}
