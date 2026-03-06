package nl.rijksoverheid.edi.wallet.platform_support

import android.content.Context
import androidx.annotation.VisibleForTesting
import nl.rijksoverheid.edi.wallet.platform_support.attested_key.AttestedKeyBridge
import nl.rijksoverheid.edi.wallet.platform_support.iso180135.Iso180135Bridge
import nl.rijksoverheid.edi.wallet.platform_support.keystore.encryption.EncryptionKeyBridge
import nl.rijksoverheid.edi.wallet.platform_support.keystore.signing.SigningKeyBridge
import nl.rijksoverheid.edi.wallet.platform_support.utilities.UtilitiesBridge
import nl.rijksoverheid.edi.wallet.platform_support.utilities.storage.StoragePathProviderImpl
import uniffi.platform_support.initPlatformSupport

class PlatformSupport private constructor(context: Context) {

    @VisibleForTesting
    val encryptionKeyBridge = EncryptionKeyBridge(context)

    @VisibleForTesting
    val signingKeyBridge = SigningKeyBridge(context)

    @VisibleForTesting
    val attestedKeyBridge = AttestedKeyBridge(context)

    @VisibleForTesting
    val utilitiesBridge = UtilitiesBridge(StoragePathProviderImpl(context))

    @VisibleForTesting
    val iso180135Bridge = Iso180135Bridge(context)

    init {
        initPlatformSupport(signingKeyBridge, encryptionKeyBridge, attestedKeyBridge, utilitiesBridge, iso180135Bridge)
    }

    companion object {
        @Volatile
        private var INSTANCE: PlatformSupport? = null

        fun getInstance(context: Context): PlatformSupport =
            INSTANCE ?: synchronized(this) {
                INSTANCE ?: PlatformSupport(context.applicationContext).also { INSTANCE = it }
            }
    }

}
