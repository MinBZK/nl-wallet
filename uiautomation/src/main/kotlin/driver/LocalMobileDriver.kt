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
        index = index,
    )

    private fun createDriver(
        platform: Platform,
        deviceName: String,
        platformVersion: String,
        automationName: String,
        udid: String,
        index: Int?,
    ): AppiumDriver {
        val serviceUrl = AppiumServiceProvider.service?.url
            ?: throw IllegalStateException("Appium service not started — call AppiumServiceProvider.startService() first")
        val bundleIdSuffix = if (index != null) ".$index" else ""

        return when (platform) {
            Platform.ANDROID -> AndroidDriver(serviceUrl, buildAndroidOptions().apply {
                if (deviceName.isNotBlank()) setDeviceName(deviceName)
                setPlatformName("Android")
                if (platformVersion.isNotBlank()) setPlatformVersion(platformVersion)
                setAutomationName(automationName)
                if (udid.isNotBlank()) setCapability("appium:udid", udid)
            }).also { driver ->
                // Suppress Chrome's first-run screen so deepLink navigation lands on the target URL.
                // Chrome reads '/data/local/tmp/chrome-command-line' at startup
                // For details see https://www.chromium.org/developers/how-tos/run-chromium-with-flags/
                driver.executeScript(
                    "mobile: shell",
                    mapOf("command" to "sh -c 'echo chrome --disable-fre --no-default-browser-check --no-first-run --disable-notifications > /data/local/tmp/chrome-command-line'")
                )
            }
            Platform.IOS -> IOSDriver(serviceUrl, buildIOSOptions(
                updatedWDABundleId = "nl.ictu.edi.wallet.web-driver-agent-runner$bundleIdSuffix",
                wdaLocalPort = if (index != null) 8099 + index else null,
            ).apply {
                if (deviceName.isNotBlank()) setDeviceName(deviceName)
                if (platformVersion.isNotBlank()) setPlatformVersion(platformVersion)
                if (udid.isNotBlank()) setCapability("appium:udid", udid)
            })
        }
    }
}
