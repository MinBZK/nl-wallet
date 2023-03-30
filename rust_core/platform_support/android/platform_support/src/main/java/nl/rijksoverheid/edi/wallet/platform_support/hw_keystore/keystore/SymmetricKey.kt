// Inspired by IRMAMobile: https://github.com/privacybydesign/irmamobile/blob/v6.4.1/android/app/src/main/java/foundation/privacybydesign/irmamobile/irma_mobile_bridge/ECDSA.java
package nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.keystore

import android.content.Context
import uniffi.platform_support.EncryptionKeyBridge
import java.security.KeyStore

private const val KEYSTORE_PROVIDER = "AndroidKeyStore"

class SymmetricKey(private val context: Context, private val keyAlias: String) :
    EncryptionKeyBridge {

    private val keyStore: KeyStore = KeyStore.getInstance(KEYSTORE_PROVIDER)

    init {
        keyStore.load(null)
    }

    override fun encrypt(payload: List<UByte>): List<UByte> {
        TODO("Implement encrypt")
    }

    override fun decrypt(payload: List<UByte>): List<UByte> {
        TODO("Implement decrypt")
    }

}
