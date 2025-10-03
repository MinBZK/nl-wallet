package driver

import com.codeborne.selenide.WebDriverProvider
import data.TestConfigRepository.Companion.testConfig
import io.appium.java_client.android.AndroidDriver
import io.appium.java_client.ios.IOSDriver
import org.openqa.selenium.Capabilities
import org.openqa.selenium.WebDriver
import org.openqa.selenium.remote.DesiredCapabilities
import java.net.URL
import java.util.Locale

class BrowserStackMobileDriver : WebDriverProvider {

    override fun createDriver(capabilities: Capabilities): WebDriver {
        val caps = DesiredCapabilities()

        return when (testConfig.platformName.lowercase(Locale.US)) {
            "android" -> AndroidDriver(URL(BROWSER_STACK_SERVER_URL), caps)
            "ios"     -> IOSDriver(URL(BROWSER_STACK_SERVER_URL), caps)
            else      -> throw IllegalArgumentException("Invalid platform name: ${testConfig.platformName}")
        }
    }

    companion object {
        private const val BROWSER_STACK_SERVER_URL = "https://hub-cloud.browserstack.com/wd/hub"
    }
}
