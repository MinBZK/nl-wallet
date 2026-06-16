package nl.rijksoverheid.edi.wallet

import android.app.Application
import io.sentry.SentryOptions
import io.sentry.android.core.SentryAndroid

class WalletApplication : Application() {

    override fun onCreate() {
        super.onCreate()
        initializeSentry()
    }

    private fun initializeSentry() {
        if (BuildConfig.SENTRY_DSN.isBlank()) return

        SentryAndroid.init(this) { options ->
            options.dsn = BuildConfig.SENTRY_DSN
            options.environment = BuildConfig.SENTRY_ENVIRONMENT.ifBlank { "unspecified" }
            options.release = BuildConfig.SENTRY_RELEASE.ifBlank { null }
            options.isDebug = BuildConfig.DEBUG
            options.setSendDefaultPii(false)
            options.setAnrEnabled(true)
            options.setEnableNdk(true)
            options.setAttachAnrThreadDump(true)
            options.setTombstoneEnabled(true)
            options.setAttachRawTombstone(true)
            options.setReportHistoricalTombstones(true)
            options.setBeforeSend(SentryOptions.BeforeSendCallback { event, _ ->
                event.user?.geo = null
                event.user?.ipAddress = null
                event
            })
        }
    }
}
