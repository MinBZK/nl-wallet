package nl.rijksoverheid.edi.wallet.platform_support

import android.content.Context
import androidx.annotation.VisibleForTesting
import nl.rijksoverheid.edi.wallet.platform_support.attested_key.AttestedKeyBridge
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

    // TODO: PVW-4069: Don't hard-code google cloud project number here
    @VisibleForTesting
    val attestedKeyBridge = AttestedKeyBridge(context, 12143997365u)

    @VisibleForTesting
    val utilitiesBridge = UtilitiesBridge(StoragePathProviderImpl(context))

    init {
        initPlatformSupport(signingKeyBridge, encryptionKeyBridge, attestedKeyBridge, utilitiesBridge)
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
