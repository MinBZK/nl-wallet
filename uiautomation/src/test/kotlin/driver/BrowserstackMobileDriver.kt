package driver

import com.codeborne.selenide.WebDriverProvider
import config.TestDataConfig.Companion.browserstackAccessKey
import config.TestDataConfig.Companion.browserstackUserName
import config.TestDataConfig.Companion.testDataConfig
import helper.Browserstack
import org.openqa.selenium.Capabilities
import org.openqa.selenium.WebDriver
import org.openqa.selenium.remote.DesiredCapabilities
import org.openqa.selenium.remote.RemoteWebDriver
import uiTests.TestBase

import util.SetupTestTagHandler

import java.net.URL

class BrowserstackMobileDriver : WebDriverProvider {

    override fun createDriver(capabilities: Capabilities): WebDriver {
        val remoteDevice = testDataConfig.defaultRemoteDevice
            ?: throw UninitializedPropertyAccessException("Make sure 'device' in testDataConfig resolves to a browserStackDevice")

        val caps = DesiredCapabilities()
        val browserstackOptions = HashMap<String, Any>()

        // Set other BrowserStack capabilities
        browserstackOptions["projectName"] = testDataConfig.browserStackCapabilities.project
        browserstackOptions["appiumVersion"] = "2.0.1"
        browserstackOptions["disableAnimations"] = "true"
        browserstackOptions["buildName"] = Browserstack.buildName
        browserstackOptions["sessionName"] = TestBase.sessionName
        browserstackOptions["idleTimeout"] = BROWSER_STACK_IDLE_TIMEOUT_SECONDS
        caps.setCapability("bstack:options", browserstackOptions)

        // Specify device and os_version for testing
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
                "android" -> Browserstack.getAppUrl("NLWalletAndroid")
                "ios" -> Browserstack.getAppUrl("NLWalletIos")
                else -> throw IllegalArgumentException("Invalid app: ${remoteDevice.platformName}")
            },
        )

        return RemoteWebDriver(
            URL("http://$browserstackUserName:$browserstackAccessKey@hub-cloud.browserstack.com/wd/hub"),
            caps,
        )
    }

    companion object {
        private const val APPIUM_RETRY_BACKOFF_TIME_MILLIS = 500
        private const val BROWSER_STACK_IDLE_TIMEOUT_SECONDS = 60 // Default: 90 seconds
    }
}
