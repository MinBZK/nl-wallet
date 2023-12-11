package service

import io.appium.java_client.service.local.AppiumDriverLocalService
import io.appium.java_client.service.local.AppiumServiceBuilder
import io.appium.java_client.service.local.flags.GeneralServerFlag

object AppiumServiceProvider {
    var service: AppiumDriverLocalService? = null

    fun startService() {
        if (service != null) throw UnsupportedOperationException("Service already running!")

        val environment = HashMap<String, String>()
        environment["PATH"] = "/usr/local/bin:" + System.getenv("PATH")

        val serviceBuilder = AppiumServiceBuilder()
            .usingAnyFreePort() // Use any port, in case the default 4723 is already taken
            .withArgument(GeneralServerFlag.ALLOW_INSECURE, "chromedriver_autodownload")
            .withArgument(GeneralServerFlag.DEBUG_LOG_SPACING)
            .withArgument(GeneralServerFlag.LOG_LEVEL, "info")
            .withArgument(GeneralServerFlag.RELAXED_SECURITY)
            .withArgument(GeneralServerFlag.SESSION_OVERRIDE)
            .withEnvironment(environment)

        service = AppiumDriverLocalService.buildService(serviceBuilder)
        service?.start()
    }

    fun stopService() {
        service?.stop()
        service = null
    }
}
