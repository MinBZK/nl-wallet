package nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.ecdsa

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import kotlin.text.Charsets.US_ASCII

@RunWith(AndroidJUnit4::class)
class ECDSAKeyInstrumentedTest {

    companion object {
        private const val key1Identifier = "key1"
        private const val key2Identifier = "key2"
    }


    private lateinit var ecdsaKeyStore: ECDSAKeyStore

    @Before
    fun setup() {
        val instrumentationContext = InstrumentationRegistry.getInstrumentation().context
        ecdsaKeyStore = ECDSAKeyStore(instrumentationContext)
    }

    @Test
    fun test_init() {
        val key1 = ecdsaKeyStore.getOrCreateKey(key1Identifier)
        val key1again = ecdsaKeyStore.getOrCreateKey(key1Identifier)
        assertNotEquals(
            "Keys with same identifier are wrapped in different objects",
            key1,
            key1again
        )
    }

    @Test
    fun test_pub_key() {
        val key1 = ecdsaKeyStore.getOrCreateKey(key1Identifier) as ECDSAKey
        val key1again = ecdsaKeyStore.getOrCreateKey(key1Identifier) as ECDSAKey
        val key2 = ecdsaKeyStore.getOrCreateKey(key2Identifier) as ECDSAKey
        assertEquals(
            "Keys with the same identifier should be equal",
            key1.publicKey(),
            key1again.publicKey()
        )
        assertNotEquals(
            "Keys with a different identifier should not be equal",
            key1.publicKey(),
            key2.publicKey()
        )
    }

    @Test
    fun test_sign() {
        val key1 = ecdsaKeyStore.getOrCreateKey(key1Identifier) as ECDSAKey
        val key1again = ecdsaKeyStore.getOrCreateKey(key1Identifier) as ECDSAKey
        val key2 = ecdsaKeyStore.getOrCreateKey(key2Identifier) as ECDSAKey


        val message = "This is a message that will be signed."

        val emptySignature = key1.sign(emptyList())
        val signature1 = key1.sign(message.toByteArray(charset = US_ASCII).map { it.toUByte() })
        val signature1Repeat =
            key1.sign(message.toByteArray(charset = US_ASCII).map { it.toUByte() })
        val signature1Again =
            key1again.sign(message.toByteArray(charset = US_ASCII).map { it.toUByte() })
        val signature2 = key2.sign(message.toByteArray(charset = US_ASCII).map { it.toUByte() })

        assertTrue("An empty payload should produce a signature", emptySignature.size > 0)
        assertNotEquals(
            "Signatures signed with the same key instance should differ",
            signature1,
            signature1Repeat
        )
        assertNotEquals(
            "Signatures signed with the same key should differ",
            signature1,
            signature1Again
        )
        assertNotEquals(
            "Signatures signed with a different key should differ",
            signature1,
            signature2
        )
    }
}
