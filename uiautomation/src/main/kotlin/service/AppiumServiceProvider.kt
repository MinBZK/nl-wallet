package service

import io.appium.java_client.service.local.AppiumDriverLocalService
import io.appium.java_client.service.local.AppiumServiceBuilder
import io.appium.java_client.service.local.flags.GeneralServerFlag

object AppiumServiceProvider {
    var service: AppiumDriverLocalService? = null
        private set

    fun startService(sessionOverride: Boolean = true) {
        if (service != null) return
        service = AppiumDriverLocalService.buildService(
            AppiumServiceBuilder()
                .usingAnyFreePort()
                .withArgument(GeneralServerFlag.ALLOW_INSECURE, "chromedriver_autodownload")
                .withArgument(GeneralServerFlag.DEBUG_LOG_SPACING)
                .withArgument(GeneralServerFlag.LOG_LEVEL, "info")
                .withArgument(GeneralServerFlag.RELAXED_SECURITY)
                // SESSION_OVERRIDE omitted when false — two concurrent sessions must coexist
                .apply { if (sessionOverride) withArgument(GeneralServerFlag.SESSION_OVERRIDE) }
        )?.also { it.start() } ?: throw IllegalStateException("Failed to build Appium service")
        Runtime.getRuntime().addShutdownHook(Thread { stopService() })
    }

    fun stopService() {
        service?.stop()
        service = null
    }
}
