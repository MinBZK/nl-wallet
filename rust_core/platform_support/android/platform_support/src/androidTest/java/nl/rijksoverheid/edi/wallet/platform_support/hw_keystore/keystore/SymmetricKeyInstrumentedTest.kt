package nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.keystore

import androidx.test.ext.junit.runners.AndroidJUnit4
import nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.util.toByteArray
import nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.util.toUByteList
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class SymmetricKeyInstrumentedTest {

    private lateinit var hwKeyStoreBridge: HwKeyStoreBridge

    @Before
    fun setup() {
        hwKeyStoreBridge = HwKeyStoreBridge.bridge as HwKeyStoreBridge
    }

    @After
    fun cleanup() {
        //FIXME: Currently cleaning the keystore causes any recurring test to fail, likely due to some
        //FIXME: magic in the EncryptedFile implementation (which ideally we replace alltogether)
//        hwKeyStoreBridge.clean()
    }

    @Test
    fun test_init() {
        hwKeyStoreBridge.getOrCreateSymmetricKey()
    }

    @Test
    fun test_encrypt_decrypt() {
        val key = hwKeyStoreBridge.getOrCreateSymmetricKey()
        val originalMessage = "Hi there!".toByteArray()
        val encryptedMessage = key.encrypt(originalMessage.toUByteList())
        assertNotEquals("Encrypted message should not match the original", originalMessage, encryptedMessage)
        val decryptedMessage = key.decrypt(encryptedMessage)
        assertEquals("Decrypted message should match the original", String(originalMessage), String(decryptedMessage.toByteArray()))
    }
}