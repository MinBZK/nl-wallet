package nl.rijksoverheid.edi.wallet.platform_support.utilities

import android.util.Log
import nl.rijksoverheid.edi.wallet.platform_support.BuildConfig

internal object PlatformLogger {
    private val enabled: Boolean
        get() = BuildConfig.DEBUG || BuildConfig.ALLOW_RELEASE_LOGS

    fun d(tag: String, message: String) {
        if (enabled) Log.d(tag, message)
    }

    fun w(tag: String, message: String, throwable: Throwable? = null) {
        if (!enabled) return
        if (throwable == null) {
            Log.w(tag, message)
        } else {
            Log.w(tag, message, throwable)
        }
    }

    fun e(tag: String, message: String) {
        if (enabled) Log.e(tag, message)
    }
}
