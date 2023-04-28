package drivers

import com.codeborne.selenide.WebDriverProvider
import config.TestDataConfig.Companion.testDataConfig
import io.appium.java_client.AppiumDriver
import io.appium.java_client.android.options.UiAutomator2Options
import io.appium.java_client.ios.options.XCUITestOptions
import org.openqa.selenium.Capabilities
import org.openqa.selenium.WebDriver
import server.AppiumServiceProvider

class LocalMobileDriver : WebDriverProvider {

    override fun createDriver(capabilities: Capabilities): WebDriver {
        AppiumServiceProvider.startService()
        val localDevice = testDataConfig.defaultLocalDevice
            ?: throw UninitializedPropertyAccessException("Make sure 'device' in testDataConfig resolves to a localDevice")

        // Set Android or iOS specific capabilities
        val options = if (localDevice.platformName == "android") {
            UiAutomator2Options().apply {
                setAppPackage(testDataConfig.appPackage)
                setAppActivity(testDataConfig.appActivity)
                ignoreHiddenApiPolicyError()
            }
        } else {
            XCUITestOptions().apply {
                setBundleId(testDataConfig.bundleId)
            }
        }
        options.merge(capabilities)

        // Set other capabilities
        options.setAutomationName("Flutter")
        options.setPlatformName(localDevice.platformName)
        options.setDeviceName(localDevice.deviceName)
        options.setPlatformVersion(localDevice.platformVersion)

        // Initialise the local Webdriver
        // and desired capabilities defined above
        return AppiumDriver(AppiumServiceProvider.server?.url, options)
    }
}