package nl.rijksoverheid.edi.wallet.platform_support.utilities

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import nl.rijksoverheid.edi.wallet.platform_support.PlatformSupport
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class UtilitiesBridgeInstrumentedTest {

    @Test
    fun bridge_is_initialized() {
        val context = InstrumentationRegistry.getInstrumentation().context
        val platformSupport = PlatformSupport.getInstance(context)
        assertNotNull(platformSupport.utilitiesBridge)
    }

    @Test
    fun bridge_test_storage_path() {
        // Explicitly load platform_support since utilities_test_storage_path() is stripped from rust_core
        System.loadLibrary("platform_support")

        // The Rust code will panic if this test fails.
        utilities_test_storage_path()
    }

    companion object {
        @JvmStatic
        external fun utilities_test_storage_path()
    }
}
