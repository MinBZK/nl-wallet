package nl.rijksoverheid.edi.wallet.platform_support.attested_key

import android.security.keystore.KeyProperties
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.newSingleThreadContext
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import nl.rijksoverheid.edi.wallet.platform_support.PlatformSupport
import nl.rijksoverheid.edi.wallet.platform_support.keystore.signing.SIGNATURE_ALGORITHM
import nl.rijksoverheid.edi.wallet.platform_support.utilities.toByteArray
import nl.rijksoverheid.edi.wallet.platform_support.utilities.toUByteList
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.fail
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.platform_support.AttestationData
import uniffi.platform_support.AttestedKeyException
import uniffi.platform_support.AttestedKeyType
import java.io.ByteArrayInputStream
import java.security.KeyFactory
import java.security.Signature
import java.security.cert.CertificateFactory
import java.security.cert.X509Certificate
import java.security.spec.X509EncodedKeySpec

@RunWith(AndroidJUnit4::class)
@DelicateCoroutinesApi // Needed for `newSingleThreadContext`
@ExperimentalCoroutinesApi // Needed for `newSingleThreadContext`, `Dispatchers.setMain` and `Dispatchers.resetMain`
class AttestedKeyBridgeInstrumentedTest {
    companion object {
        const val CHALLENGE: String = "test-challenge"
        const val GOOGLE_CLOUD_PROJECT_NUMBER: ULong = 12143997365u

        @JvmStatic
        external fun attested_key_test()
    }

    private lateinit var attestedKeyBridge: AttestedKeyBridge

    private val mainThreadSurrogate = newSingleThreadContext("UI thread")

    private val keyFactory: KeyFactory = KeyFactory.getInstance(KeyProperties.KEY_ALGORITHM_EC)
    private val certFactory: CertificateFactory = CertificateFactory.getInstance("X.509")

    @Before
    fun setUp() {
        Dispatchers.setMain(mainThreadSurrogate)

        val context = InstrumentationRegistry.getInstrumentation().context
        attestedKeyBridge = PlatformSupport.getInstance(context).attestedKeyBridge
    }

    @After
    fun tearDown() {
        attestedKeyBridge.clean()

        Dispatchers.resetMain() // reset the main dispatcher to the original Main dispatcher
        mainThreadSurrogate.close()
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
        val challenge = CHALLENGE.toByteArray().toUByteList()

        // Generate a new key using `attest`
        val attestationData = attestedKeyBridge.attest(id, challenge, GOOGLE_CLOUD_PROJECT_NUMBER)

        // Verify that attestationData is an instance of `Google`
        if (attestationData is AttestationData.Google) {
            // Note that appAttestationToken is encrypted and is not decrypted
            // by the wallet_app, but by the backend (wallet_provider). Hence,
            // here, we only check that the response is not empty.
            assert(attestationData.appAttestationToken.isNotEmpty())
            // Verify that the certificate chain is not empty
            assert(attestationData.certificateChain.size >= 2) {
                "expected at least the root certificate and the key's certificate"
            }
            // Check that the certificate's public key equals the public key as obtained from the attestedKeyBridge
            val certificate = parseCertificate(attestationData.certificateChain[0])
            val publicKeyBytes = certificate.publicKey.encoded.toUByteList()
            assertEquals(publicKeyBytes, attestedKeyBridge.publicKey(id))
        } else {
            fail("This should never occur on Android")
        }
    }

    @Test
    fun test_attest_for_existing_key_should_fail() = runTest {
        val id = "id"
        val challenge = CHALLENGE.toByteArray().toUByteList()

        attestedKeyBridge.attest(id, challenge, GOOGLE_CLOUD_PROJECT_NUMBER)
        assertFails<AttestedKeyException.Other>(
            "reason=precondition failed: A key already exists with alias: `ecdsa_id`"
        ) {
            attestedKeyBridge.attest(id, challenge, GOOGLE_CLOUD_PROJECT_NUMBER)
        }
    }

    @Test
    fun test_delete() = runTest {
        val id = "id"
        val challenge = CHALLENGE.toByteArray().toUByteList()

        // Verify public key for 'id' does not exist
        assertFails<AttestedKeyException.Other>("reason=precondition failed: Key not found for alias: `ecdsa_id`") {
            attestedKeyBridge.publicKey(id)
        }

        // Generate new key via `attest`
        attestedKeyBridge.attest(id, challenge, GOOGLE_CLOUD_PROJECT_NUMBER)

        // Verify public key for 'id' does exist
        val publicKeyBytes = attestedKeyBridge.publicKey(id).toByteArray()
        // Verify publicKey is an X.509 Ecdsa key
        val x509EncodedKeySpec = X509EncodedKeySpec(publicKeyBytes)
        val publicKey = keyFactory.generatePublic(x509EncodedKeySpec)
        assertEquals("X.509", publicKey.format)
        assertEquals("EC", publicKey.algorithm)

        // Delete key
        attestedKeyBridge.delete(id)

        // Verify public key for 'id' does no longer exist
        assertFails<AttestedKeyException.Other>("reason=precondition failed: Key not found for alias: `ecdsa_id`") {
            attestedKeyBridge.publicKey(id)
        }
    }

    @Test
    fun test_delete_should_succeed_when_key_does_not_exist() = runTest {
        val id = "id"

        // Verify public key for 'id' does not exist
        assertFails<AttestedKeyException.Other>("reason=precondition failed: Key not found for alias: `ecdsa_id`") {
            attestedKeyBridge.publicKey(id)
        }

        // Delete key for 'id'
        attestedKeyBridge.delete(id)
    }

    @Test
    fun test_sign() = runTest {
        val id = "id"
        val challenge = CHALLENGE.toByteArray().toUByteList()
        val valueToSign = "value to sign".toByteArray().toUByteList()

        // Generate a new key
        attestedKeyBridge.attest(id, challenge, GOOGLE_CLOUD_PROJECT_NUMBER)

        // Sign the valueToSign
        val signature = attestedKeyBridge.sign(id, valueToSign)

        // Verify the signature
        assert(
            isValidSignature(
                signature.toByteArray(), valueToSign.toByteArray(), attestedKeyBridge.publicKey(id).toByteArray()
            )
        )
    }

    @Test
    fun test_sign_should_fail_for_non_existing_key() = runTest {
        val id = "id"
        val valueToSign = "value to sign".toByteArray().toUByteList()

        assertFails<AttestedKeyException.Other>("reason=precondition failed: Key not found for alias: `ecdsa_id`") {
            attestedKeyBridge.sign(id, valueToSign)
        }
    }

   @Test
   fun bridge_test_attested_key() {
       // Explicitly load platform_support since hw_keystore_test_hardware_signature() is stripped from rust_core
       System.loadLibrary("platform_support")

       // The Rust code will panic if this test fails.
       attested_key_test()
   }

    private fun parseCertificate(certificateBytes: List<UByte>): X509Certificate {
        val certificateInputStream = ByteArrayInputStream(certificateBytes.toByteArray())
        val certificate = certFactory.generateCertificate(certificateInputStream)
        return certificate as X509Certificate
    }

    private fun isValidSignature(
        signatureBytes: ByteArray,
        payload: ByteArray,
        publicKeyBytes: ByteArray,
    ): Boolean {
        val x509EncodedKeySpec = X509EncodedKeySpec(publicKeyBytes)
        val publicKey = keyFactory.generatePublic(x509EncodedKeySpec)
        val signature = Signature.getInstance(SIGNATURE_ALGORITHM)
        signature.initVerify(publicKey)
        signature.update(payload)
        return signature.verify(signatureBytes)
    }
}


private inline fun <reified T> assertFails(
    expectedMessage: String? = null,
    crossinline block: suspend () -> Unit
) {
    runBlocking {
        try {
            block()
            fail("Expected exception, but got none")
        } catch (e: Exception) {
            when (e) {
                is T -> expectedMessage?.let {
                    assertEquals(it, e.message)
                }

                else -> fail("Expected exception ${T::class.qualifiedName}, but got ${e::class.qualifiedName}")
            }
        }
    }
}
