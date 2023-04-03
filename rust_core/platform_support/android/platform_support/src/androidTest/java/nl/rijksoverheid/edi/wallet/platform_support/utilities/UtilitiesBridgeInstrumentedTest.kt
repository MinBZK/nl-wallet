package nl.rijksoverheid.edi.wallet.platform_support.utilities

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class UtilitiesBridgeInstrumentedTest {

    @Before
    fun setUp() {
        System.loadLibrary("platform_support")
    }

    @Test
    fun utilities_test_file_dir() {
        //TODO: Implement this test
        Assert.assertTrue(true)
    }
}