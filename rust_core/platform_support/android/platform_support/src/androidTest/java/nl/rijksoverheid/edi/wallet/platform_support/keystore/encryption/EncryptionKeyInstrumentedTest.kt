package nl.rijksoverheid.edi.wallet.platform_support.keystore.encryption

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.After
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Class that verifies that a [EncryptionKey] can be properly
 * instantiated. Encrypt/Decrypt funcitonality is tested
 * through [EncryptionKeyBridgeInstrumentedTest]
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
}