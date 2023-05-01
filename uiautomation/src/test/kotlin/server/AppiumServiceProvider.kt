package server

import io.appium.java_client.service.local.AppiumDriverLocalService
import io.appium.java_client.service.local.AppiumServiceBuilder
import io.appium.java_client.service.local.flags.GeneralServerFlag

object AppiumServiceProvider {
    var server: AppiumDriverLocalService? = null

    fun startService() {
        if(server != null) throw UnsupportedOperationException("Server already running!")
        val serviceBuilder = AppiumServiceBuilder()
        // Use any port, in case the default 4723 is already taken (maybe by another Appium server)

        val environment = HashMap<String, String>()
        environment["PATH"] = "/usr/local/bin:" + System.getenv("PATH")
        serviceBuilder.usingAnyFreePort()
            .withArgument(GeneralServerFlag.SESSION_OVERRIDE)
            .withArgument(GeneralServerFlag.LOG_LEVEL, "debug")
            .withArgument(GeneralServerFlag.DEBUG_LOG_SPACING)
            .withArgument(GeneralServerFlag.RELAXED_SECURITY)
            .withEnvironment(environment)
        server = AppiumDriverLocalService.buildService(serviceBuilder)
        server?.start()
    }

    fun stopServer() {
        server?.stop()
        server = null
    }
}