package driver

import com.codeborne.selenide.WebDriverProvider
import config.TestDataConfig.Companion.testDataConfig
import io.appium.java_client.AppiumDriver
import io.appium.java_client.android.options.UiAutomator2Options
import io.appium.java_client.ios.options.XCUITestOptions
import org.openqa.selenium.Capabilities
import org.openqa.selenium.WebDriver
import server.AppiumServiceProvider
import util.SetupTestTagHandler

class LocalMobileDriver : WebDriverProvider {

    private val apkPath = "../wallet_app/build/app/outputs/flutter-apk/app-profile.apk"
    private val ipaPath = "../nl.ict.edi.wallet.latest-0.1.0.ipa"

    override fun createDriver(capabilities: Capabilities): WebDriver {
        AppiumServiceProvider.startService()
        val localDevice = testDataConfig.defaultLocalDevice
            ?: throw UninitializedPropertyAccessException("Make sure 'device' in testDataConfig resolves to a localDevice")

        // Set Android or iOS specific capabilities
        val options = when (localDevice.platformName) {
            "android" -> UiAutomator2Options().apply {
                setAppPackage(testDataConfig.appPackage)
                setAppActivity(testDataConfig.appActivity)
                setApp(apkPath)
                ignoreHiddenApiPolicyError()
            }
            "ios" -> XCUITestOptions().apply {
                setBundleId(testDataConfig.bundleId)
                setApp(ipaPath)
            }
            else -> throw IllegalArgumentException("Invalid platformName: ${localDevice.platformName}")
        }
        options.merge(capabilities)

        // Set other capabilities
        options.setAutomationName("Flutter")
        options.setPlatformName(localDevice.platformName)
        options.setDeviceName(localDevice.deviceName)
        options.setPlatformVersion(localDevice.platformVersion)
        options.setLanguage(SetupTestTagHandler.language)
        options.setLocale(SetupTestTagHandler.locale)

        // Initialise the local Webdriver
        // and desired capabilities defined above
        return AppiumDriver(AppiumServiceProvider.server?.url, options)
    }
}
