package com.example.platform_support.hw_keystore

import androidx.test.ext.junit.runners.AndroidJUnit4

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
