package com.example.platform_support.hwkeystore

import android.util.Log
import com.example.platform_support.hwkeystore.bridge.PlatformKeyStore
import uniffi.hw_keystore.KeyStoreBridge
import uniffi.hw_keystore.initHwKeystore

class HWKeyStore {
    companion object {
        val shared = HWKeyStore()

        private val keyStore: KeyStoreBridge = PlatformKeyStore()

        init {
            initHwKeystore(bridge = keyStore)

            //TESTING
            val key = keyStore.getOrCreateKey("first_key_id")
            Log.d(">>> HWKeyStore", ">>> SigningKey: $key")
        }
    }
}
