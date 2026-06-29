package nl.rijksoverheid.edi.wallet

import android.app.Application
import io.sentry.Breadcrumb
import io.sentry.SentryEvent
import io.sentry.SentryLevel
import io.sentry.SentryOptions
import io.sentry.android.core.SentryAndroid

class WalletApplication : Application() {

    companion object {
        private const val SENTRY_MAX_BREADCRUMBS = 25
        private val BREADCRUMB_CATEGORIES = setOf("wallet.flow", "wallet.native")
        private val BREADCRUMB_MESSAGE_PATTERN = Regex("^[a-z0-9_]+(\\.[a-z0-9_]+)*$")
    }

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
            options.setMaxBreadcrumbs(SENTRY_MAX_BREADCRUMBS)
            options.setBeforeBreadcrumb(SentryOptions.BeforeBreadcrumbCallback { breadcrumb, _ ->
                breadcrumb.takeIf { it.isCuratedWalletBreadcrumb() }?.sanitized()
            })
            options.setBeforeSend(SentryOptions.BeforeSendCallback { event, _ ->
                event.user?.geo = null
                event.user?.ipAddress = null
                event.scrubBreadcrumbs()
                event
            })
        }
    }

    private fun SentryEvent.scrubBreadcrumbs() {
        breadcrumbs = breadcrumbs?.mapNotNull { breadcrumb ->
            breadcrumb.takeIf { it.isCuratedWalletBreadcrumb() }?.sanitized()
        }
    }

    private fun Breadcrumb.isCuratedWalletBreadcrumb(): Boolean =
        category in BREADCRUMB_CATEGORIES && message?.let { BREADCRUMB_MESSAGE_PATTERN.matches(it) } == true

    private fun Breadcrumb.sanitized(): Breadcrumb = apply {
        data.clear()
        level = SentryLevel.INFO
        type = "default"
    }
}
