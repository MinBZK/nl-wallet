package nl.rijksoverheid.edi.wallet.platform_support.hw_keystore

import android.content.Context
import androidx.startup.Initializer

class HWKeystoreInitializer : Initializer<HWKeyStore> {
    override fun create(context: Context): HWKeyStore = HWKeyStore(context)

    override fun dependencies(): List<Class<out Initializer<*>>> = emptyList()
}