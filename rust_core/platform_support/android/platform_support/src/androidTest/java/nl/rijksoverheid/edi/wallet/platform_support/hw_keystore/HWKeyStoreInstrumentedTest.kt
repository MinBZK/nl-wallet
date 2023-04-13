package nl.rijksoverheid.edi.wallet.platform_support.hw_keystore

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class HWKeyStoreInstrumentedTest {

    @Before
    fun setUp() {
        System.loadLibrary("platform_support")
    }

    @Test
    fun hwKeyStore_isInitialised() {
        assertNotNull(HWKeyStore.bridge)
    }

    @Test
    fun hwKeyStore_testSignature() {
        assertTrue(hw_keystore_test_hardware_signature())
    }

    companion object {
        @JvmStatic
        external fun hw_keystore_test_hardware_signature(): Boolean
    }
}
