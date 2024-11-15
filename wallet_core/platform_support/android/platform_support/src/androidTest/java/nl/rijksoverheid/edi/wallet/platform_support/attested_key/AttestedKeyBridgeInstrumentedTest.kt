package nl.rijksoverheid.edi.wallet.platform_support.attested_key

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
        val alias = "ecdsa-id"
        val challenge = "challenge".toByteArray().toUByteList()
        val attestationData = attestedKeyBridge.attest(id, challenge)
        if (attestationData is AttestationData.Google) {
            // assertEquals(attestedKeyBridge.sign(id, challenge), attestationData.appAttestationToken)
            SigningKey(alias).validate(challenge, attestationData.appAttestationToken)
        } else {
            assert(false) { "This should never occur on Android" }
        }
    }

//    @Test
//    fun bridge_test_attested_key() = runTest {
//        // Explicitly load platform_support since hw_keystore_test_hardware_signature() is stripped from rust_core
//        System.loadLibrary("platform_support")
//
//        // The Rust code will panic if this test fails.
//        attested_key_test()
//    }
}
