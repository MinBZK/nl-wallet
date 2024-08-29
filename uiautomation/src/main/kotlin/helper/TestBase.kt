package helper

import com.codeborne.selenide.Configuration
import com.codeborne.selenide.Selenide
import data.TestConfigRepository.Companion.testConfig
import driver.BrowserStackMobileDriver
import driver.LocalMobileDriver
import org.junit.jupiter.api.AfterAll
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeAll
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.extension.ExtendWith
import service.AppiumServiceProvider
import util.TestInfoHandler.Companion.processTestInfo

@ExtendWith(TestResultsListener::class)
open class TestBase {

    @BeforeEach
    fun startDriver(testInfo: TestInfo) {
        // Process session name, platform, language and locale
        processTestInfo(testInfo)

        // Start driver
        Selenide.open()
    }

    @AfterEach
    fun closeDriver() {
        // Close browser tab
        try {
            Selenide.closeWindow()
        } catch (e: Exception) {
            // Ignore
        }

        // Close web driver
        if (!testConfig.remote) {
            Selenide.closeWebDriver()
        }
    }

    companion object {
        const val MAX_RETRY_COUNT = 3

        @JvmStatic
        @BeforeAll
        fun setup() {
            // Start Appium service if running locally
            if (!testConfig.remote) {
                AppiumServiceProvider.startService()
            }

            if (testConfig.remote) {
                Configuration.browser = BrowserStackMobileDriver::class.java.name
            } else {
                Configuration.browser = LocalMobileDriver::class.java.name
            }

            Configuration.browserSize = null
        }

        @JvmStatic
        @AfterAll
        fun destroy() {
            // Stop Appium service if running locally
            if (!testConfig.remote) {
                AppiumServiceProvider.stopService()
            }
        }
    }
}
