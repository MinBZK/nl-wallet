package com.example.platform_support.hw_keystore

import androidx.test.ext.junit.runners.AndroidJUnit4

import org.junit.Test
import org.junit.runner.RunWith

import org.junit.Assert.assertNotNull

/**
 * Instrumented test, which will execute on an Android device.
 *
 * See [testing documentation](http://d.android.com/tools/testing).
 */
@RunWith(AndroidJUnit4::class)
class HWKeyStoreInstrumentedTest {
    @Test
    fun hwKeyStore_isInitialised() {
        val hwKeyStore = HWKeyStore.shared
        assertNotNull(hwKeyStore)
    }
}
