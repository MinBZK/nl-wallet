package driver

import com.codeborne.selenide.WebDriverProvider
import data.TestConfigRepository.Companion.testConfig
import helper.BrowserStackHelper
import io.appium.java_client.android.AndroidDriver
import org.openqa.selenium.Capabilities
import org.openqa.selenium.WebDriver
import org.openqa.selenium.remote.DesiredCapabilities
import util.EnvironmentUtil
import util.TestInfoHandler
import java.net.URL

class BrowserStackMobileDriver : WebDriverProvider {

    private val browserStackUserName = EnvironmentUtil.getVar("BROWSERSTACK_USER")
    private val browserStackAccessKey = EnvironmentUtil.getVar("BROWSERSTACK_KEY")
    private val sessionName = EnvironmentUtil.getVar("CI_JOB_ID").takeIf { it.isNotBlank() } ?: "local--run"

    override fun createDriver(capabilities: Capabilities): WebDriver {

        // Specify device and OS version for testing
        val caps = DesiredCapabilities()
        caps.setCapability("appium:newCommandTimeout", 120)
        caps.setCapability("appium:automationName", "Flutter")
        caps.setCapability("platformName", testConfig.platformName)
        caps.setCapability("appium:platformVersion", testConfig.platformVersion)
        caps.setCapability("appium:deviceName", testConfig.deviceName)
        caps.setCapability("appium:language", TestInfoHandler.language)
        caps.setCapability("appium:locale", TestInfoHandler.locale)
        caps.setCapability("appium:retryBackoffTime", APPIUM_RETRY_BACKOFF_TIME_MILLIS)
        caps.setCapability("appium:disableSuppressAccessibilityService", APPIUM_DISABLE_SUPPRESS_ACCESSIBILITY_SERVICE)
        caps.setCapability("appium:autoGrantPermissions", true)

        // Set other BrowserStack capabilities
        val browserstackOptions = HashMap<String, Any>()
        browserstackOptions["appiumVersion"] = "2.6.0"
        browserstackOptions["buildName"] = sessionName
        browserstackOptions["disableAnimations"] = "true"
        browserstackOptions["idleTimeout"] = BROWSER_STACK_IDLE_TIMEOUT_SECONDS
        browserstackOptions["networkLogs"] = "true"
        browserstackOptions["sessionName"] = TestInfoHandler.sessionName
        caps.setCapability("bstack:options", browserstackOptions)

        // Set URL of the application under test
        caps.setCapability(
            "appium:app",
            when (testConfig.platformName) {
                "android" -> BrowserStackHelper.getAppUrl(
                    BROWSER_STACK_RECENT_APPS_ENDPOINT,
                    browserStackUserName,
                    browserStackAccessKey,
                    CUSTOM_ID_PREFIX_ANDROID_APP + testConfig.appIdentifier,
                )

                "ios" -> BrowserStackHelper.getAppUrl(
                    BROWSER_STACK_RECENT_APPS_ENDPOINT,
                    browserStackUserName,
                    browserStackAccessKey,
                    CUSTOM_ID_PREFIX_IOS_APP + testConfig.appIdentifier,
                )

                else -> throw IllegalArgumentException("Invalid platform name: ${testConfig.platformName}")
            },
        )

        return AndroidDriver(
            URL("http://$browserStackUserName:$browserStackAccessKey@$BROWSER_STACK_SERVER_URL"),
            caps,
        )
    }

    companion object {
        private const val CUSTOM_ID_PREFIX_ANDROID_APP = "NLWalletAndroid_"
        private const val CUSTOM_ID_PREFIX_IOS_APP = "NLWalletIos_"

        private const val BROWSER_STACK_SERVER_URL = "hub-cloud.browserstack.com/wd/hub"
        private const val BROWSER_STACK_RECENT_APPS_ENDPOINT =
            "https://api-cloud.browserstack.com/app-automate/recent_apps/"
        private const val BROWSER_STACK_IDLE_TIMEOUT_SECONDS = 60 // Default: 90 seconds
        private const val APPIUM_RETRY_BACKOFF_TIME_MILLIS = 500 // Default: 3000 milliseconds
        private const val APPIUM_DISABLE_SUPPRESS_ACCESSIBILITY_SERVICE = false
    }
}
