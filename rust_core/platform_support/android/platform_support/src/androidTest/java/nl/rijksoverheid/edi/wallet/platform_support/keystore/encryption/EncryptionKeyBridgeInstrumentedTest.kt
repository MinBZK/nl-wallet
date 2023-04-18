package nl.rijksoverheid.edi.wallet.platform_support.keystore.encryption

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import nl.rijksoverheid.edi.wallet.platform_support.PlatformSupport
import nl.rijksoverheid.edi.wallet.platform_support.util.toByteArray
import nl.rijksoverheid.edi.wallet.platform_support.util.toUByteList
import org.junit.After
import org.junit.Assert.*
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class EncryptionKeyBridgeInstrumentedTest {

    private lateinit var encryptionKeyBridge: EncryptionKeyBridge

    companion object {
        const val KEY_ID = "key1"

        @JvmStatic
        external fun hw_keystore_test_hardware_encryption(): Boolean
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
            encryptionKeyBridge.encrypt(KEY_ID, originalMessage.toUByteList()).toByteArray()
        assertNotEquals(
            "Encrypted message should not match the original",
            originalMessage,
            encryptedMessage
        )
        val decryptedMessage = encryptionKeyBridge.decrypt(KEY_ID, encryptedMessage.toUByteList())
        assertEquals(
            "Decrypted message should match the original", String(originalMessage),
            String(decryptedMessage.toByteArray())
        )
    }

    @Test
    fun test_long_encrypt_decrypt() {
        val originalMessage = "Hello World, Repeated!".repeat(1024).toByteArray()
        val encryptedMessage =
            encryptionKeyBridge.encrypt(KEY_ID, originalMessage.toUByteList()).toByteArray()
        assertNotEquals(
            "Encrypted message should not match the original",
            originalMessage,
            encryptedMessage
        )
        val decryptedMessage = encryptionKeyBridge.decrypt(KEY_ID, encryptedMessage.toUByteList())
        assertEquals(
            "Decrypted message should match the original", String(originalMessage),
            String(decryptedMessage.toByteArray())
        )
    }

    @Test
    fun bridge_test_symmetric_encryption() {
        // Explicitly load platform_support since hw_keystore_test_hardware_encryption() is stripped from rust_core
        System.loadLibrary("platform_support")

        assertTrue(
            "Could not complete encryption round trip",
            hw_keystore_test_hardware_encryption()
        )
    }
}