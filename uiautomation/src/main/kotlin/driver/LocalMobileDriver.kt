package driver

import com.codeborne.selenide.WebDriverProvider
import config.TestDataConfig.Companion.testDataConfig
import io.appium.java_client.android.AndroidDriver
import io.appium.java_client.android.options.UiAutomator2Options
import io.appium.java_client.ios.options.XCUITestOptions
import org.openqa.selenium.Capabilities
import org.openqa.selenium.WebDriver
import service.AppiumServiceProvider
import util.SetupTestTagHandler

class LocalMobileDriver : WebDriverProvider {

    private val apkPath = "../wallet_app/build/app/outputs/flutter-apk/app-profile.apk"
    private val ipaPath = "../nl.ict.edi.wallet.latest-0.1.0.ipa"

    override fun createDriver(capabilities: Capabilities): WebDriver {
        val localDevice = testDataConfig.defaultLocalDevice
            ?: throw UninitializedPropertyAccessException("Make sure 'device' in testDataConfig resolves to a localDevice")

        // Set Android or iOS specific capabilities
        val options = when (localDevice.platformName) {
            "android" -> UiAutomator2Options().apply {
                setApp(apkPath)
                setAppActivity(testDataConfig.appActivity)
                setAppPackage(testDataConfig.appPackage)
                setIgnoreHiddenApiPolicyError(true)
            }

            "ios" -> XCUITestOptions().apply {
                setApp(ipaPath)
                setBundleId(testDataConfig.bundleId)
            }

            else -> throw IllegalArgumentException("Invalid platformName: ${localDevice.platformName}")
        }
        options.merge(capabilities)

        // Set other capabilities
        options.setAutomationName("Flutter")
        options.setDeviceName(localDevice.deviceName)
        options.setLanguage(SetupTestTagHandler.language)
        options.setLocale(SetupTestTagHandler.locale)
        options.setPlatformName(localDevice.platformName)
        options.setPlatformVersion(localDevice.platformVersion)

        // Initialise the local WebDriver with desired capabilities defined above
        //TODO: Add switch between AndroidDriver and IOSDriver
        return AndroidDriver(AppiumServiceProvider.service?.url, options)
    }
}
