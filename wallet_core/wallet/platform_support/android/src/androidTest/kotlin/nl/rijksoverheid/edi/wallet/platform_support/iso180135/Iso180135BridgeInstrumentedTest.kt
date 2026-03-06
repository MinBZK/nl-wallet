package nl.rijksoverheid.edi.wallet.platform_support.iso180135

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import nl.rijksoverheid.edi.wallet.platform_support.PlatformSupport
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Iso180135BridgeInstrumentedTest {
    @Test
    fun bridge_test_start_qr_handover() {
        // Explicitly load platform_support since iso180135_test_start_qr_handover() is stripped from rust_core
        System.loadLibrary("platform_support")

        // The Rust code will panic if this test fails.
        iso18013_5_test_start_qr_handover()
    }

    companion object {
        @JvmStatic
        external fun iso18013_5_test_start_qr_handover()
    }
}
