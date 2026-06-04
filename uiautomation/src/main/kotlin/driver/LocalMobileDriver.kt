package driver

import data.TestConfigRepository.Companion.testConfig
import domain.DeviceCapabilities
import io.appium.java_client.AppiumDriver
import io.appium.java_client.android.AndroidDriver
import io.appium.java_client.ios.IOSDriver
import service.AppiumServiceProvider

object LocalMobileDriver {

    fun createDriver(): AppiumDriver = createDriver(
        platformName = testConfig.platformName,
        deviceName = testConfig.deviceName,
        platformVersion = testConfig.platformVersion,
        automationName = testConfig.automationName,
        udid = testConfig.udid,
        index = null,
    )

    fun createDriver(device: DeviceCapabilities, index: Int): AppiumDriver = createDriver(
        platformName = device.platformName,
        deviceName = device.deviceName,
        platformVersion = device.platformVersion,
        automationName = when (device.platformName.lowercase()) {
            "android" -> "UiAutomator2"
            "ios" -> "XCUITest"
            else -> throw IllegalArgumentException("Unsupported platform: ${device.platformName}")
        },
        udid = device.udid,
        index = index,
    )

    private fun createDriver(
        platformName: String,
        deviceName: String,
        platformVersion: String,
        automationName: String,
        udid: String,
        index: Int?,
    ): AppiumDriver {
        val serviceUrl = AppiumServiceProvider.service?.url
            ?: throw IllegalStateException("Appium service not started — call AppiumServiceProvider.startService() first")
        val bundleIdSuffix = if (index != null) ".$index" else ""

        return when (platformName.lowercase()) {
            "android" -> AndroidDriver(serviceUrl, buildAndroidOptions().apply {
                if (deviceName.isNotBlank()) setDeviceName(deviceName)
                setPlatformName("Android")
                if (platformVersion.isNotBlank()) setPlatformVersion(platformVersion)
                setAutomationName(automationName)
                if (udid.isNotBlank()) setCapability("appium:udid", udid)
            })
            "ios" -> IOSDriver(serviceUrl, buildIOSOptions(
                updatedWDABundleId = "nl.ictu.edi.wallet.web-driver-agent-runner$bundleIdSuffix",
                wdaLocalPort = if (index != null) 8099 + index else null,
            ).apply {
                if (deviceName.isNotBlank()) setDeviceName(deviceName)
                if (platformVersion.isNotBlank()) setPlatformVersion(platformVersion)
                if (udid.isNotBlank()) setCapability("appium:udid", udid)
            })
            else -> throw IllegalArgumentException("Invalid platform name: $platformName")
        }
    }
}
