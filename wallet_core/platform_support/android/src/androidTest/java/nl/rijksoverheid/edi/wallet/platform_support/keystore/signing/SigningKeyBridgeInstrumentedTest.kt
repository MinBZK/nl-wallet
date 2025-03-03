package nl.rijksoverheid.edi.wallet.platform_support.keystore.signing

import android.security.keystore.KeyProperties
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import nl.rijksoverheid.edi.wallet.platform_support.PlatformSupport
import nl.rijksoverheid.edi.wallet.platform_support.utilities.toByteArray
import nl.rijksoverheid.edi.wallet.platform_support.utilities.toUByteList
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import java.security.KeyFactory
import java.security.Signature
import java.security.spec.X509EncodedKeySpec
import kotlin.text.Charsets.US_ASCII

@RunWith(AndroidJUnit4::class)
class SigningKeyBridgeInstrumentedTest {

    companion object {
        private const val KEY_1_IDENTIFIER = "key1"
        private const val KEY_2_IDENTIFIER = "key2"

        @JvmStatic
        external fun hw_keystore_test_hardware_signature()
    }


    private lateinit var signingKeyBridge: SigningKeyBridge

    @Before
    fun setup() {
        val context = InstrumentationRegistry.getInstrumentation().context
        signingKeyBridge = PlatformSupport.getInstance(context).signingKeyBridge
    }

    @After
    fun cleanup() {
        signingKeyBridge.clean()
    }

    @Test
    fun test_init() {
        assertNotNull("SigningKeyBridge should be initialized", signingKeyBridge)
    }

    @Test
    fun test_pub_key() {
        assertEquals(
            "Keys with the same identifier should be equal",
            signingKeyBridge.publicKey(KEY_1_IDENTIFIER),
            signingKeyBridge.publicKey(KEY_1_IDENTIFIER)
        )
        assertNotEquals(
            "Keys with a different identifier should not be equal",
            signingKeyBridge.publicKey(KEY_1_IDENTIFIER),
            signingKeyBridge.publicKey(KEY_2_IDENTIFIER)
        )
    }

    @Test
    fun test_sign() {
        val message = "This is a message that will be signed."

        val emptySignature = signingKeyBridge.sign(KEY_1_IDENTIFIER, emptyList())
        val signature1 = signingKeyBridge.sign(
            KEY_1_IDENTIFIER,
            message.toByteArray(charset = US_ASCII).toUByteList()
        )
        val signature1Again =
            signingKeyBridge.sign(
                KEY_1_IDENTIFIER,
                message.toByteArray(charset = US_ASCII).toUByteList()
            )
        val signature2 = signingKeyBridge.sign(
            KEY_2_IDENTIFIER,
            message.toByteArray(charset = US_ASCII).toUByteList()
        )

        assertTrue("An empty payload should produce a signature", emptySignature.isNotEmpty())
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

    @Test
    fun test_verify_signature() {
        val message = "This is a message that will be signed."

        val signature1 = signingKeyBridge.sign(
            KEY_1_IDENTIFIER,
            message.toByteArray(charset = US_ASCII).toUByteList()
        )
        val signature1Repeat =
            signingKeyBridge.sign(
                KEY_1_IDENTIFIER,
                message.toByteArray(charset = US_ASCII).toUByteList()
            )

        assertTrue(
            "Signature should be valid",
            isValidSignature(
                signature1.toByteArray(),
                message.toByteArray(),
                signingKeyBridge.publicKey(KEY_1_IDENTIFIER).toByteArray()
            )
        )
        assertTrue(
            "Signature should be valid",
            isValidSignature(
                signature1Repeat.toByteArray(),
                message.toByteArray(),
                signingKeyBridge.publicKey(KEY_1_IDENTIFIER).toByteArray()
            )
        )
    }

    @Test
    fun test_signature_mismatch() {
        val message = "This is a message that will be signed."
        val otherMessage = "Some other message"
        assertNotEquals(
            "Messages used to verify signature mismatch should not be equal",
            message,
            otherMessage
        )

        val messageSignature = signingKeyBridge.sign(
            KEY_1_IDENTIFIER,
            message.toByteArray(charset = US_ASCII).toUByteList()
        )

        assertFalse(
            "Signature from different key should not be valid",
            isValidSignature(
                messageSignature.toByteArray(),
                message.toByteArray(),
                signingKeyBridge.publicKey(KEY_2_IDENTIFIER).toByteArray()
            )
        )
        assertFalse(
            "Signature with different payload should not be valid",
            isValidSignature(
                messageSignature.toByteArray(),
                otherMessage.toByteArray(),
                signingKeyBridge.publicKey(KEY_1_IDENTIFIER).toByteArray()
            )
        )
    }

    @Test
    fun test_key_deletion() {
        val keyAPublicKey = signingKeyBridge.getOrCreateKey(KEY_1_IDENTIFIER).publicKey()
        signingKeyBridge.delete(KEY_1_IDENTIFIER)
        val keyBPublicKey = signingKeyBridge.getOrCreateKey(KEY_1_IDENTIFIER).publicKey()
        assertNotEquals(
            "Keys with same identifier should be different after intermediate deletion",
            keyAPublicKey,
            keyBPublicKey
        )
    }

    @Test
    fun bridge_test_signature() {
        // Explicitly load platform_support since hw_keystore_test_hardware_signature() is stripped from rust_core
        System.loadLibrary("platform_support")

        // The Rust code will panic if this test fails.
        hw_keystore_test_hardware_signature()
    }

    private fun isValidSignature(
        signatureBytes: ByteArray,
        payload: ByteArray,
        publicKeyBytes: ByteArray,
    ): Boolean {
        val x509EncodedKeySpec = X509EncodedKeySpec(publicKeyBytes)
        val keyFactory: KeyFactory = KeyFactory.getInstance(KeyProperties.KEY_ALGORITHM_EC)
        val publicKey = keyFactory.generatePublic(x509EncodedKeySpec)
        val signature = Signature.getInstance(SIGNATURE_ALGORITHM)
        signature.initVerify(publicKey)
        signature.update(payload)
        return signature.verify(signatureBytes)
    }
}
