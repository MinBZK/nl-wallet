package driver

import com.codeborne.selenide.WebDriverProvider
import config.TestDataConfig.Companion.browserstackAccessKey
import config.TestDataConfig.Companion.browserstackUserName
import config.TestDataConfig.Companion.testDataConfig
import helper.BrowserStackHelper
import helper.TestBase
import io.appium.java_client.android.AndroidDriver
import org.openqa.selenium.Capabilities
import org.openqa.selenium.WebDriver
import org.openqa.selenium.remote.DesiredCapabilities
import util.SetupTestTagHandler
import java.net.URL

class BrowserStackMobileDriver : WebDriverProvider {

    override fun createDriver(capabilities: Capabilities): WebDriver {
        val remoteDevice = testDataConfig.defaultRemoteDevice
            ?: throw UninitializedPropertyAccessException("Make sure 'device' in testDataConfig resolves to a browserStackDevice")

        val caps = DesiredCapabilities()
        val browserstackOptions = HashMap<String, Any>()

        // Set other BrowserStack capabilities
        browserstackOptions["appiumVersion"] = "2.0.1"
        browserstackOptions["buildName"] = BrowserStackHelper.buildName
        browserstackOptions["disableAnimations"] = "true"
        browserstackOptions["idleTimeout"] = BROWSER_STACK_IDLE_TIMEOUT_SECONDS
        browserstackOptions["networkLogs"] = "true"
        browserstackOptions["projectName"] = testDataConfig.browserStackCapabilities.project
        browserstackOptions["sessionName"] = TestBase.sessionName
        caps.setCapability("bstack:options", browserstackOptions)

        // Specify device and OS version for testing
        caps.setCapability("platformName", remoteDevice.platformName)
        caps.setCapability("appium:automationName", "Flutter")
        caps.setCapability("appium:platformVersion", remoteDevice.platformVersion)
        caps.setCapability("appium:deviceName", remoteDevice.deviceName)
        caps.setCapability("appium:language", SetupTestTagHandler.language)
        caps.setCapability("appium:locale", SetupTestTagHandler.locale)
        caps.setCapability("appium:retryBackoffTime", APPIUM_RETRY_BACKOFF_TIME_MILLIS)

        // Set URL of the application under test
        caps.setCapability(
            "appium:app",
            when (remoteDevice.platformName) {
                "android" -> BrowserStackHelper.getAppUrl("NLWalletAndroid")
                "ios" -> BrowserStackHelper.getAppUrl("NLWalletIos")
                else -> throw IllegalArgumentException("Invalid app: ${remoteDevice.platformName}")
            },
        )

        return AndroidDriver(
            URL("http://$browserstackUserName:$browserstackAccessKey@hub-cloud.browserstack.com/wd/hub"),
            caps,
        )
    }

    companion object {
        private const val APPIUM_RETRY_BACKOFF_TIME_MILLIS = 500 // Default: 3000 milliseconds
        private const val BROWSER_STACK_IDLE_TIMEOUT_SECONDS = 60 // Default: 90 seconds
    }
}
