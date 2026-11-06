package driver

import data.TestConfigRepository.Companion.testConfig
import domain.DeviceCapabilities
import domain.Platform
import io.appium.java_client.AppiumDriver
import io.appium.java_client.android.AndroidDriver
import io.appium.java_client.ios.IOSDriver
import service.AppiumServiceProvider

object LocalMobileDriver {

    fun createDriver(): AppiumDriver = createDriver(
        platform = testConfig.platform,
        deviceName = testConfig.deviceName,
        platformVersion = testConfig.platformVersion,
        automationName = testConfig.automationName,
        udid = testConfig.udid,
        appPath = testConfig.appPath,
        index = null,
    )

    fun createDriver(device: DeviceCapabilities, index: Int): AppiumDriver = createDriver(
        platform = device.platform,
        deviceName = device.deviceName,
        platformVersion = device.platformVersion,
        automationName = when (device.platform) {
            Platform.ANDROID -> "UiAutomator2"
            Platform.IOS -> "XCUITest"
        },
        udid = device.udid,
        appPath = device.appPath,
        index = index,
    )

    private fun createDriver(
        platform: Platform,
        deviceName: String,
        platformVersion: String,
        automationName: String,
        udid: String,
        appPath: String,
        index: Int?,
    ): AppiumDriver {
        val serviceUrl = AppiumServiceProvider.service?.url
            ?: throw IllegalStateException("Appium service not started — call AppiumServiceProvider.startService() first")

        return when (platform) {
            Platform.ANDROID -> AndroidDriver(serviceUrl, buildAndroidOptions(appPath).apply {
                if (deviceName.isNotBlank()) setDeviceName(deviceName)
                setPlatformName("Android")
                if (platformVersion.isNotBlank()) setPlatformVersion(platformVersion)
                setAutomationName(automationName)
                if (udid.isNotBlank()) setCapability("appium:udid", udid)
            }).also { driver ->
                // Suppress Chrome's first-run screen so deepLink navigation lands on the target URL.
                // Chrome reads '/data/local/tmp/chrome-command-line' at startup, on user builds only as debug app
                // For details see https://www.chromium.org/developers/how-tos/run-chromium-with-flags/
                val chromeSetup = listOf(
                    "echo chrome --disable-fre --no-default-browser-check --no-first-run --disable-notifications > /data/local/tmp/chrome-command-line",
                    "am set-debug-app --persistent com.android.chrome",
                    "pm grant com.android.chrome android.permission.POST_NOTIFICATIONS",
                    "am force-stop com.android.chrome",
                ).joinToString("; ")
                driver.executeScript("mobile: shell", mapOf("command" to "sh -c '$chromeSetup'"))
            }
            Platform.IOS -> IOSDriver(serviceUrl, buildIOSOptions(
                appPath = appPath,
                updatedWDABundleId = "nl.ictu.edi.wallet.web-driver-agent-runner",
                wdaLocalPort = if (index != null) 8099 + index else null,
            ).apply {
                if (deviceName.isNotBlank()) setDeviceName(deviceName)
                if (platformVersion.isNotBlank()) setPlatformVersion(platformVersion)
                if (udid.isNotBlank()) setCapability("appium:udid", udid)
            })
        }
    }
}
