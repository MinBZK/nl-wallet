package nl.rijksoverheid.edi.wallet.hwkeystore.bridge

import uniffi.hw_keystore.SigningKeyBridge

class SigningKey : SigningKeyBridge {
    override fun publicKey(): List<UByte> {
        val temp = "publicKey"
        return temp.toByteArray().toList() as List<UByte>
    }

    override fun sign(payload: List<UByte>): List<UByte> {
        val temp = "sign"
        return temp.toByteArray().toList() as List<UByte>
    }
}
