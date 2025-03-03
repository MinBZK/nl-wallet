package nl.rijksoverheid.edi.wallet.platform_support.keystore.encryption

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import nl.rijksoverheid.edi.wallet.platform_support.PlatformSupport
import nl.rijksoverheid.edi.wallet.platform_support.utilities.toByteArray
import nl.rijksoverheid.edi.wallet.platform_support.utilities.toUByteList
import org.junit.After
import org.junit.Assert.*
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import javax.crypto.BadPaddingException

@RunWith(AndroidJUnit4::class)
class EncryptionKeyBridgeInstrumentedTest {

    private lateinit var encryptionKeyBridge: EncryptionKeyBridge

    companion object {
        const val KEY_1_IDENTIFIER = "key1"

        @JvmStatic
        external fun hw_keystore_test_hardware_encryption()
    }

    @Before
    fun setup() {
        val context = InstrumentationRegistry.getInstrumentation().context
        encryptionKeyBridge = PlatformSupport.getInstance(context).encryptionKeyBridge
    }

    @After
    fun cleanup() {
        encryptionKeyBridge.clean()
    }

    @Test
    fun test_init() {
        assertNotNull("Should be initialized", encryptionKeyBridge)
    }

    @Test
    fun test_encrypt_decrypt() {
        val originalMessage = "Hello World!".toByteArray()
        val encryptedMessage =
            encryptionKeyBridge.encrypt(KEY_1_IDENTIFIER, originalMessage.toUByteList())
                .toByteArray()
        assertNotEquals(
            "Encrypted message should not match the original",
            originalMessage,
            encryptedMessage
        )
        val decryptedMessage =
            encryptionKeyBridge.decrypt(KEY_1_IDENTIFIER, encryptedMessage.toUByteList())
        assertEquals(
            "Decrypted message should match the original", String(originalMessage),
            String(decryptedMessage.toByteArray())
        )
    }

    @Test
    fun test_long_encrypt_decrypt() {
        val originalMessage = "Hello World, Repeated!".repeat(1024).toByteArray()
        val encryptedMessage =
            encryptionKeyBridge.encrypt(KEY_1_IDENTIFIER, originalMessage.toUByteList())
                .toByteArray()
        assertNotEquals(
            "Encrypted message should not match the original",
            originalMessage,
            encryptedMessage
        )
        val decryptedMessage =
            encryptionKeyBridge.decrypt(KEY_1_IDENTIFIER, encryptedMessage.toUByteList())
        assertEquals(
            "Decrypted message should match the original", String(originalMessage),
            String(decryptedMessage.toByteArray())
        )
    }

    @Test
    fun test_key_deletion() {
        val originalMessage = "Hello World!".toByteArray()
        /// Encrypt with a newly generated key
        val encryptedMessage =
            encryptionKeyBridge.encrypt(KEY_1_IDENTIFIER, originalMessage.toUByteList())
                .toByteArray()
        /// Delete the key used to encrypt
        encryptionKeyBridge.delete(KEY_1_IDENTIFIER)
        /// Decrypt with a (presumably) newly generated key, assuming it was deleted correctly
        assertThrows("Decrypting message with different key should throw",
            BadPaddingException::class.java
        ) { encryptionKeyBridge.decrypt(KEY_1_IDENTIFIER, encryptedMessage.toUByteList()) }
    }

    @Test
    fun bridge_test_symmetric_encryption() {
        // Explicitly load platform_support since hw_keystore_test_hardware_encryption() is stripped from rust_core
        System.loadLibrary("platform_support")

        // The Rust code will panic if this test fails.
        hw_keystore_test_hardware_encryption()
    }
}
