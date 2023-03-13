package com.example.platform_support.hwkeystore.bridge

import android.util.Log
import com.example.platform_support.hwkeystore.keystore.KeyStoreKey
import uniffi.hw_keystore.SigningKeyBridge

class SigningKey(private val key: KeyStoreKey) : SigningKeyBridge {

    override fun publicKey(): List<UByte> {
        Log.d("SigningKey.publicKey()", ">>> $key")

        val temp = "publicKey"
        return temp.toByteArray().toList() as List<UByte>
    }

    override fun sign(payload: List<UByte>): List<UByte> {
        Log.d("SigningKey.sign()", ">>> $key")

        val temp = "sign"
        return temp.toByteArray().toList() as List<UByte>
    }
}
