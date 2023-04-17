package nl.rijksoverheid.edi.wallet.platform_support.utilities.storage

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class StoragePathProviderInstrumentedTest {

    private lateinit var storagePathProvider: StoragePathProvider

    @Before
    fun setUp() {
        val context = InstrumentationRegistry.getInstrumentation().context
        storagePathProvider = StoragePathProviderImpl(context)
    }

    @Test
    fun test_path_is_valid() {
        assertNotNull(storagePathProvider.getStoragePath())
        // Verify that the `/*/files` folder is provided.
        assertTrue("^/.*/files\$".toRegex().matches(storagePathProvider.getStoragePath()))
    }
}
