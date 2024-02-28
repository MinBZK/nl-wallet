package driver

import com.codeborne.selenide.WebDriverProvider
import data.TestConfigRepository.Companion.testConfig
import io.appium.java_client.android.AndroidDriver
import io.appium.java_client.android.options.UiAutomator2Options
import io.appium.java_client.ios.options.XCUITestOptions
import org.openqa.selenium.Capabilities
import org.openqa.selenium.WebDriver
import service.AppiumServiceProvider
import util.TestInfoHandler

class LocalMobileDriver : WebDriverProvider {

    private val apkPath = "../wallet_app/build/app/outputs/flutter-apk/app-profile.apk"
    private val ipaPath = "../nl.ict.edi.wallet.latest-0.1.0.ipa"

    override fun createDriver(capabilities: Capabilities): WebDriver {

        // Set Android or iOS specific capabilities
        val options = when (testConfig.platformName) {
            "android" -> UiAutomator2Options().apply {
                setApp(apkPath)
                setAppPackage(testConfig.appIdentifier)
                setIgnoreHiddenApiPolicyError(true)
            }

            "ios" -> XCUITestOptions().apply {
                setApp(ipaPath)
                setBundleId(testConfig.appIdentifier)
            }

            else -> throw IllegalArgumentException("Invalid platform name: ${testConfig.platformName}")
        }
        options.merge(capabilities)

        // Set other capabilities
        options.setAutomationName("Flutter")
        options.setDeviceName(testConfig.deviceName)
        options.setPlatformName(testConfig.platformName)
        options.setPlatformVersion(testConfig.platformVersion)
        options.setLanguage(TestInfoHandler.language)
        options.setLocale(TestInfoHandler.locale)

        // Initialise the local WebDriver with desired capabilities defined above
        return AndroidDriver(AppiumServiceProvider.service?.url, options)
    }
}
