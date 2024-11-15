package nl.rijksoverheid.edi.wallet.platform_support.attested_key

import android.security.keystore.KeyProperties
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.newSingleThreadContext
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import nl.rijksoverheid.edi.wallet.platform_support.PlatformSupport
import nl.rijksoverheid.edi.wallet.platform_support.keystore.signing.SigningKey
import nl.rijksoverheid.edi.wallet.platform_support.util.toUByteList
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertNotNull
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.platform_support.AttestationData
import uniffi.platform_support.AttestedKeyType
import kotlinx.coroutines.ExperimentalCoroutinesApi
import nl.rijksoverheid.edi.wallet.platform_support.keystore.signing.SIGNATURE_ALGORITHM
import nl.rijksoverheid.edi.wallet.platform_support.util.toByteArray
import org.junit.Assert.fail
import uniffi.platform_support.AttestedKeyException
import java.security.KeyFactory
import java.security.Signature
import java.security.spec.X509EncodedKeySpec

@RunWith(AndroidJUnit4::class)
@ExperimentalCoroutinesApi
class AttestedKeyBridgeInstrumentedTest {
    companion object {
        @JvmStatic
        external fun attested_key_test()
    }

    private lateinit var attestedKeyBridge: AttestedKeyBridge

    @ExperimentalCoroutinesApi
    private val mainThreadSurrogate = newSingleThreadContext("UI thread")

    @Before
    fun setUp() {
        Dispatchers.setMain(mainThreadSurrogate)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain() // reset the main dispatcher to the original Main dispatcher
        mainThreadSurrogate.close()
    }

    @Before
    fun setup() {
        val context = InstrumentationRegistry.getInstrumentation().context
        attestedKeyBridge = PlatformSupport.getInstance(context).attestedKeyBridge
    }

    @After
    fun cleanup() {
        attestedKeyBridge.clean()
    }

    @Test
    fun test_init() {
        assertNotNull("SigningKeyBridge should be initialized", attestedKeyBridge)
    }

    @Test
    fun test_keyType_is_google() {
        assertEquals(AttestedKeyType.GOOGLE, attestedKeyBridge.keyType())
    }

    @Test
    fun test_generate_returns_different_ids() = runTest {
        val id1 = attestedKeyBridge.generate()
        val id2 = attestedKeyBridge.generate()
        assertNotNull(id1)
        assertNotNull(id2)
        assertNotEquals(id1, id2)
    }

    @Test
    fun test_attest() = runTest {
        val id = "id"
        val challenge = "challenge".toByteArray().toUByteList()

        // Generate a new key using `attest`
        val attestationData = attestedKeyBridge.attest(id, challenge)

        // Verify that attestationData is an instance of `Google`
        if (attestationData is AttestationData.Google) {
            // Verify the attestation token is a valid signature of the challenge
            assert(
                isValidSignature(
                    attestationData.appAttestationToken.toByteArray(),
                    challenge.toByteArray(),
                    attestedKeyBridge.publicKey(id).toByteArray()
                )
            )
            // Verify that the certificate chain is not empty
            assert(attestationData.certificateChain.isNotEmpty()) { "expected a certificate chain" }
        } else {
            fail("This should never occur on Android")
        }
    }

    @Test
    fun test_delete() = runTest {
        val id = "id"
        val challenge = "challenge".toByteArray().toUByteList()

        // Verify public key for 'id' does not exist
        try {
            attestedKeyBridge.publicKey(id)
            fail("Should raise an exception")
        } catch (e: AttestedKeyException) {
            assert(e is AttestedKeyException.Other)
            assertEquals(
                "reason=precondition failed: Key not found for alias: `ecdsa-id`",
                e.message
            )
        }

        // Generate new key via `attest`
        attestedKeyBridge.attest(id, challenge)

        // Verify public key for 'id' does exist
        attestedKeyBridge.publicKey(id)

        // Delete key
        attestedKeyBridge.delete(id)

        // Verify public key for 'id' does no longer exist
        try {
            attestedKeyBridge.publicKey(id)
            fail("Should raise an exception")
        } catch (e: AttestedKeyException) {
            assert(e is AttestedKeyException.Other)
            assertEquals(
                "reason=precondition failed: Key not found for alias: `ecdsa-id`",
                e.message
            )
        }
    }

    @Test
    fun test_sign() = runTest {
        val id = "id"
        val challenge = "challenge".toByteArray().toUByteList()
        val valueToSign = "value to sign".toByteArray().toUByteList()

        // Generate a new key
        attestedKeyBridge.attest(id, challenge)

        // Sign the valueToSign
        val signature = attestedKeyBridge.sign(id, valueToSign)

        // Verify the signature
        assert(
            isValidSignature(
                signature.toByteArray(),
                valueToSign.toByteArray(),
                attestedKeyBridge.publicKey(id).toByteArray()
            )
        )
    }

//    @Test
//    fun bridge_test_attested_key() = runTest {
//        // Explicitly load platform_support since hw_keystore_test_hardware_signature() is stripped from rust_core
//        System.loadLibrary("platform_support")
//
//        // The Rust code will panic if this test fails.
//        attested_key_test()
//    }

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
