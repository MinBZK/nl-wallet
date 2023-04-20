package nl.rijksoverheid.edi.wallet.platform_support.keystore

import androidx.test.ext.junit.runners.AndroidJUnit4
import nl.rijksoverheid.edi.wallet.platform_support.util.toByteArray
import nl.rijksoverheid.edi.wallet.platform_support.util.toUByteList
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class AESKeyInstrumentedTest {

    private lateinit var hwKeyStoreBridge: HwKeyStoreBridge

    companion object {
        const val KEY_ID = "key1"
    }

    @Before
    fun setup() {
        hwKeyStoreBridge = HwKeyStoreBridge.bridge as HwKeyStoreBridge
    }

    @After
    fun cleanup() {
        hwKeyStoreBridge.clean()
    }

    @Test
    fun test_init() {
        hwKeyStoreBridge.getOrCreateEncryptionKey(KEY_ID)
    }

    @Test
    fun test_encrypt_decrypt() {
        val key = hwKeyStoreBridge.getOrCreateEncryptionKey(KEY_ID)
        val originalMessage = "Hello World!".toByteArray()
        val encryptedMessage = key.encrypt(originalMessage.toUByteList()).toByteArray()
        assertNotEquals(
            "Encrypted message should not match the original",
            originalMessage,
            encryptedMessage
        )
        val decryptedMessage = key.decrypt(encryptedMessage.toUByteList())
        assertEquals(
            "Decrypted message should match the original", String(originalMessage),
            String(decryptedMessage.toByteArray())
        )
    }

    @Test
    fun test_long_encrypt_decrypt() {
        val key = hwKeyStoreBridge.getOrCreateEncryptionKey(KEY_ID)
        val originalMessage = "Hello World, Repeated!".repeat(1024).toByteArray()
        val encryptedMessage = key.encrypt(originalMessage.toUByteList()).toByteArray()
        assertNotEquals(
            "Encrypted message should not match the original",
            originalMessage,
            encryptedMessage
        )
        val decryptedMessage = key.decrypt(encryptedMessage.toUByteList())
        assertEquals(
            "Decrypted message should match the original", String(originalMessage),
            String(decryptedMessage.toByteArray())
        )
    }
}