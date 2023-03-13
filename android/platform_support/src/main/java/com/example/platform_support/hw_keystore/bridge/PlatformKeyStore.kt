package com.example.platform_support.hw_keystore.bridge

import android.util.Log
import com.example.platform_support.hw_keystore.keystore.KeyStoreKey
import uniffi.hw_keystore.KeyStoreBridge
import uniffi.hw_keystore.SigningKeyBridge

class PlatformKeyStore : KeyStoreBridge {
    override fun getOrCreateKey(identifier: String): SigningKeyBridge {
        val key = KeyStoreKey(identifier = identifier)

        // Test encryption
        val payload = "payload".toByteArray()
        val encrypted = key.encrypt(payload, System.out)

        Log.d(">>> PlatformKeyStore", "encrypted: $encrypted")

        return SigningKey(key = key)
    }
}
