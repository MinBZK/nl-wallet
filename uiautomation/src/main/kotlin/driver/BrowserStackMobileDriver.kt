package driver

import com.codeborne.selenide.WebDriverProvider
import data.TestConfigRepository.Companion.testConfig
import domain.Platform
import io.appium.java_client.android.AndroidDriver
import io.appium.java_client.ios.IOSDriver
import org.openqa.selenium.Capabilities
import org.openqa.selenium.WebDriver
import org.openqa.selenium.remote.DesiredCapabilities
import java.net.URL

class BrowserStackMobileDriver : WebDriverProvider {

    override fun createDriver(capabilities: Capabilities): WebDriver {
        val caps = DesiredCapabilities()

        return when (testConfig.platform) {
            Platform.ANDROID -> AndroidDriver(URL(BROWSER_STACK_SERVER_URL), caps)
            Platform.IOS     -> IOSDriver(URL(BROWSER_STACK_SERVER_URL), caps)
        }
    }

    companion object {
        private const val BROWSER_STACK_SERVER_URL = "https://hub-cloud.browserstack.com/wd/hub"
    }
}
