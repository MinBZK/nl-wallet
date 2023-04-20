package nl.rijksoverheid.edi.wallet.platform_support.keystore.encryption

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import nl.rijksoverheid.edi.wallet.platform_support.util.toByteArray
import nl.rijksoverheid.edi.wallet.platform_support.util.toUByteList
import org.junit.After
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Class that verifies that a [EncryptionKey] can be properly
 * instantiated. Encrypt/Decrypt functionality is tested
 * more thoroughly in [EncryptionKeyBridgeInstrumentedTest]
 */
@RunWith(AndroidJUnit4::class)
class EncryptionKeyInstrumentedTest {

    companion object {
        const val KEY_1_ALIAS = "key1"
    }

    private val context = InstrumentationRegistry.getInstrumentation().context

    @After
    fun cleanup() {
        runCatching { EncryptionKey(KEY_1_ALIAS).delete() }
    }

    @Test(expected = IllegalArgumentException::class)
    fun test_key_throws_when_not_created() {
        EncryptionKey(KEY_1_ALIAS)
    }

    @Test
    fun test_key_available_when_created() {
        EncryptionKey.createKey(context, KEY_1_ALIAS)
        EncryptionKey(KEY_1_ALIAS)
    }

    @Test
    fun test_encrypt_decrypt_matches_original() {
        EncryptionKey.createKey(context, KEY_1_ALIAS)
        val key = EncryptionKey(KEY_1_ALIAS)
        val originalMessage = "Hello Wallet".toByteArray().toUByteList()
        val encryptedMessage = key.encrypt(originalMessage.toByteArray().toUByteList())
        assertNotEquals("Encrypted message should differ from original", originalMessage, encryptedMessage)
        val decryptedMessage = key.decrypt(encryptedMessage)
        assertEquals("Decrypted message should be the same as the original", originalMessage, decryptedMessage)
    }
}