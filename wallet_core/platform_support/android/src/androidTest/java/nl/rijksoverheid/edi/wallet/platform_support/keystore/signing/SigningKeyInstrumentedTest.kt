package nl.rijksoverheid.edi.wallet.platform_support.keystore.signing

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import nl.rijksoverheid.edi.wallet.platform_support.utilities.toUByteList
import org.junit.After
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Class that verifies that a [SigningKey] can be properly
 * instantiated. PublicKey/Sign functionality is tested
 * more thoroughly in [SigningKeyBridgeInstrumentedTest]
 */
@RunWith(AndroidJUnit4::class)
class SigningKeyInstrumentedTest {

    companion object {
        const val KEY_1_ALIAS = "key1"
    }

    private val context = InstrumentationRegistry.getInstrumentation().context

    @After
    fun cleanup() {
        runCatching { SigningKey(KEY_1_ALIAS).delete() }
    }

    @Test(expected = IllegalArgumentException::class)
    fun test_key_throws_when_not_created() {
        SigningKey(KEY_1_ALIAS)
    }

    @Test
    fun test_key_available_when_created() {
        SigningKey.createKey(context, KEY_1_ALIAS)
        SigningKey(KEY_1_ALIAS)
    }

    @Test
    fun test_public_key_available() {
        SigningKey.createKey(context, KEY_1_ALIAS)
        val key = SigningKey(KEY_1_ALIAS)
        assertTrue(key.publicKey().isNotEmpty())
    }

    @Test
    fun test_signatures_are_unique() {
        SigningKey.createKey(context, KEY_1_ALIAS)
        val key = SigningKey(KEY_1_ALIAS)
        val message = "Hello Wallet".toByteArray().toUByteList()
        val firstSignature = key.sign(message)
        val secondSignature = key.sign(message)
        assertNotEquals("Consecutive signatures should differ", firstSignature, secondSignature)
    }


}
