package nl.rijksoverheid.edi.wallet.platform_support.close_proximity_disclosure

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class CloseProximityDisclosureBridgeInstrumentedTest {
    @Test
    fun bridge_test_start_qr_handover() {
        // Explicitly load platform_support since close_proximity_disclosure_test_start_qr_handover() is stripped from rust_core
        System.loadLibrary("platform_support")

        // The Rust code will panic if this test fails.
        close_proximity_disclosure_test_start_qr_handover()
    }

    companion object {
        @JvmStatic
        external fun close_proximity_disclosure_test_start_qr_handover()
    }
}
