package driver

import com.codeborne.selenide.WebDriverProvider
import data.TestConfigRepository.Companion.testConfig
import io.appium.java_client.android.AndroidDriver
import io.appium.java_client.android.options.UiAutomator2Options
import io.appium.java_client.ios.IOSDriver
import io.appium.java_client.ios.options.XCUITestOptions
import org.openqa.selenium.Capabilities
import org.openqa.selenium.WebDriver
import service.AppiumServiceProvider
import util.TestInfoHandler

class LocalMobileDriver : WebDriverProvider {

    private val apkPath = "../nl.ictu.edi.wallet.latest-0.3.0-profile.apk"
    private val ipaPath = "../nl.ictu.edi.wallet.latest-0.3.0.ipa"

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
        options.setDeviceName(testConfig.deviceName)
        options.setPlatformName(testConfig.platformName)
        options.setPlatformVersion(testConfig.platformVersion)
        options.setLanguage(TestInfoHandler.language)
        options.setLocale(TestInfoHandler.locale)
        options.setAutomationName("Flutter")
        options.setCapability("appium:newCommandTimeout", 120)

        // Initialise the local WebDriver with desired capabilities defined above
        return when (testConfig.platformName) {
            "android" -> {
                AndroidDriver(AppiumServiceProvider.service?.url, options)
            }
            "ios" -> {
                options.setCapability("appium:autoAcceptAlerts", true)
                options.setCapability("includeSafariInWebviews", true)
                options.setCapability("udid", "");
                options.setCapability("xcodeOrgId", "");
                options.setCapability("xcodeSigningId", "");
                options.setCapability("updatedWDABundleId", "");
                options.setCapability("wdaLaunchTimeout", 60000);
                options.setCapability("wdaConnectionTimeout", 60000);
                IOSDriver(AppiumServiceProvider.service?.url, options)
            }
            else -> {
                throw IllegalArgumentException("Invalid platform name: ${testConfig.platformName}")
            }
        }
    }
}
