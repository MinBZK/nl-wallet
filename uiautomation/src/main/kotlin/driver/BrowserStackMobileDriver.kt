package driver

import com.codeborne.selenide.WebDriverProvider
import data.TestConfigRepository.Companion.testConfig
import helper.BrowserStackHelper
import io.appium.java_client.android.AndroidDriver
import io.appium.java_client.ios.IOSDriver
import org.openqa.selenium.Capabilities
import org.openqa.selenium.WebDriver
import org.openqa.selenium.remote.DesiredCapabilities
import util.EnvironmentUtil
import util.TestInfoHandler
import java.net.URL
import java.util.Locale

class BrowserStackMobileDriver : WebDriverProvider {

    private val browserStackUserName = EnvironmentUtil.getVar("BROWSERSTACK_USER")
    private val browserStackAccessKey = EnvironmentUtil.getVar("BROWSERSTACK_KEY")
    private val sessionName = EnvironmentUtil.getVar("CI_JOB_ID").takeIf { it.isNotBlank() } ?: "local--run"

    override fun createDriver(capabilities: Capabilities): WebDriver {
        val caps = DesiredCapabilities()

        caps.setCapability("appium:newCommandTimeout", 120)
        caps.setCapability("appium:automationName", "Flutter") // override via env if needed
        caps.setCapability("platformName", testConfig.platformName)
        caps.setCapability("appium:platformVersion", testConfig.platformVersion)
        caps.setCapability("appium:deviceName", testConfig.deviceName)
        caps.setCapability("appium:language", TestInfoHandler.language)
        caps.setCapability("appium:locale", TestInfoHandler.locale)

        when (testConfig.platformName.lowercase(Locale.US)) {
            "android" -> applyAndroidCaps(caps)
            "ios"     -> applyIosCaps(caps)
            else      -> throw IllegalArgumentException("Invalid platform name: ${testConfig.platformName}")
        }

        val bstack = HashMap<String, Any>()
        bstack["appiumVersion"]     = "2.18.0"
        bstack["buildName"]         = sessionName
        bstack["disableAnimations"] = "true"
        bstack["idleTimeout"]       = BROWSER_STACK_IDLE_TIMEOUT_SECONDS
        bstack["networkLogs"]       = "true"
        bstack["sessionName"]       = TestInfoHandler.sessionName
        bstack["deviceName"]        = testConfig.deviceName
        bstack["osVersion"]         = testConfig.platformVersion
        caps.setCapability("bstack:options", bstack)

        val appCustomId = when (testConfig.platformName.lowercase(Locale.US)) {
            "android" -> CUSTOM_ID_PREFIX_ANDROID_APP + testConfig.appIdentifier
            "ios"     -> CUSTOM_ID_PREFIX_IOS_APP + testConfig.appIdentifier
            else      -> throw IllegalArgumentException("Invalid platform name: ${testConfig.platformName}")
        }
        caps.setCapability(
            "appium:app",
            BrowserStackHelper.getAppUrl(
                BROWSER_STACK_RECENT_APPS_ENDPOINT,
                browserStackUserName,
                browserStackAccessKey,
                appCustomId
            )
        )

        val serverUrl = URL("https://$browserStackUserName:$browserStackAccessKey@$BROWSER_STACK_SERVER_URL")

        return when (testConfig.platformName.lowercase(Locale.US)) {
            "android" -> AndroidDriver(serverUrl, caps)
            "ios"     -> IOSDriver(serverUrl, caps)
            else      -> throw IllegalArgumentException("Invalid platform name: ${testConfig.platformName}")
        }
    }

    private fun applyAndroidCaps(caps: DesiredCapabilities) {
        caps.setCapability("appium:autoGrantPermissions", true)
        caps.setCapability("appium:retryBackoffTime", APPIUM_RETRY_BACKOFF_TIME_MILLIS)
        caps.setCapability("appium:disableSuppressAccessibilityService", APPIUM_DISABLE_SUPPRESS_ACCESSIBILITY_SERVICE)
    }

    private fun applyIosCaps(caps: DesiredCapabilities) {
        caps.setCapability("appium:autoAcceptAlerts", true)
        caps.setCapability("appium:includeSafariInWebviews", true)
    }

    companion object {
        private const val CUSTOM_ID_PREFIX_ANDROID_APP = "NLWalletAndroid_"
        private const val CUSTOM_ID_PREFIX_IOS_APP = "NLWalletIos_"

        private const val BROWSER_STACK_SERVER_URL = "hub-cloud.browserstack.com/wd/hub"
        private const val BROWSER_STACK_RECENT_APPS_ENDPOINT =
            "https://api-cloud.browserstack.com/app-automate/recent_apps/"
        private const val BROWSER_STACK_IDLE_TIMEOUT_SECONDS = 60 // Default: 90
        private const val APPIUM_RETRY_BACKOFF_TIME_MILLIS = 500 // Default: 3000
        private const val APPIUM_DISABLE_SUPPRESS_ACCESSIBILITY_SERVICE = false
    }
}
