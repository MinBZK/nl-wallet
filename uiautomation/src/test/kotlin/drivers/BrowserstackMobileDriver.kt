package drivers

import com.codeborne.selenide.WebDriverProvider
import config.TestDataConfig.Companion.browserstackAccessKey
import config.TestDataConfig.Companion.browserstackUserName
import config.TestDataConfig.Companion.testDataConfig
import helper.Browserstack
import org.openqa.selenium.Capabilities
import org.openqa.selenium.WebDriver
import org.openqa.selenium.remote.DesiredCapabilities
import org.openqa.selenium.remote.RemoteWebDriver
import util.setupTestTagHandler

import java.net.MalformedURLException
import java.net.URL

class BrowserstackMobileDriver : WebDriverProvider {

    override fun createDriver(capabilities: Capabilities): WebDriver {
        val remoteDevice = testDataConfig.defaultRemoteDevice
            ?: throw UninitializedPropertyAccessException("Make sure 'device' in testDataConfig resolves to a browserStackDevice")

        val caps = DesiredCapabilities()
        val browserstackOptions = HashMap<String, Any>()

        // Set other BrowserStack capabilities
        browserstackOptions["projectName"] = testDataConfig.browserStackCapabilities.project
        browserstackOptions["appiumVersion"] = "2.0.0"
        browserstackOptions["disableAnimations"] = "true"

        caps.setCapability("bstack:options", browserstackOptions)

        // Specify device and os_version for testing
        caps.setCapability("platformName", remoteDevice.platformName)
        caps.setCapability("appium:automationName", "Flutter")
        caps.setCapability("appium:platformVersion", remoteDevice.platformVersion)
        caps.setCapability("appium:deviceName", remoteDevice.deviceName)
        caps.setCapability("appium:language", setupTestTagHandler.language)
        caps.setCapability("appium:locale", setupTestTagHandler.locale)
        // Set URL of the application under test
        caps.setCapability(
            "appium:app", when (remoteDevice.platformName) {
                "android" -> Browserstack.getAppUrl("NLWalletAndroid")
                "ios" -> Browserstack.getAppUrl("NLWalletios")
                else -> throw IllegalArgumentException("Invalid app: ${remoteDevice.platformName}")
            }
        )

        return RemoteWebDriver(
            URL("http://$browserstackUserName:$browserstackAccessKey@hub-cloud.browserstack.com/wd/hub"),
            caps
        )
    }

    private fun getBrowserstackUrl(): URL? {
        return try {
            URL(testDataConfig.server)
        } catch (e: MalformedURLException) {
            throw RuntimeException(e)
        }
    }
}
