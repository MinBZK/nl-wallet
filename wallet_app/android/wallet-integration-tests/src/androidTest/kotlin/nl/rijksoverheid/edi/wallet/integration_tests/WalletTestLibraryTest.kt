package nl.rijksoverheid.edi.wallet.integration_tests

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.filters.RequiresDevice
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

// These adapter checks also run without the Mac reader; the disclosure suite needs real BLE peers.
@RunWith(AndroidJUnit4::class)
@RequiresDevice
class WalletTestLibraryTest : WalletDeviceTest() {
    @Test
    fun testRustUsesInitializedNativeBridge() {
        WalletTestLibrary.onRustThread {
            WalletTestLibrary.instance.close_proximity_disclosure_test_start_qr_handover()
        }
    }

    @Test
    fun testRustErrorIsReturnedAndFreed() {
        val error = WalletTestLibrary.runDisclosure(registrationCase = "invalid_case")
        assertTrue(error, error?.startsWith("unknown registration certificate case:") == true)
    }
}
