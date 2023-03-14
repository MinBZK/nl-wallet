package nl.rijksoverheid.edi.wallet.platform_support.hw_keystore

import androidx.test.ext.junit.runners.AndroidJUnit4
import nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.HWKeyStore

import org.junit.Test
import org.junit.runner.RunWith

import org.junit.Assert.assertNotNull

@RunWith(AndroidJUnit4::class)
class HWKeyStoreInstrumentedTest {
    @Test
    fun hwKeyStore_isInitialised() {
        val hwKeyStore = HWKeyStore.shared
        assertNotNull(hwKeyStore)
    }
}
