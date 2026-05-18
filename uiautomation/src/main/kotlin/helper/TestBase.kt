package helper

import com.codeborne.selenide.Selenide
import com.codeborne.selenide.WebDriverRunner
import data.TestConfigRepository.Companion.testConfig
import driver.LocalMobileDriver
import io.appium.java_client.AppiumDriver
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.extension.ExtendWith
import util.TestInfoHandler.Companion.processTestInfo

@ExtendWith(ServiceHelper::class)
open class TestBase {

    fun startDriver(testInfo: TestInfo) {
        processTestInfo(testInfo)
        if (testConfig.remote) {
            Selenide.open()
        } else {
            WebDriverRunner.setWebDriver(LocalMobileDriver.createDriver())
        }
    }

    // Drivers created with Selenide's WebDriverRunner.setWebDriver() can/should be closed with Selenide.closeWebDriver()
    @AfterEach
    fun closeDriver() {
        if (!testConfig.remote) {
            try {
                clearBrowser(WebDriverRunner.getWebDriver() as AppiumDriver)
            } catch (_: Exception) {}
        }
        Selenide.closeWebDriver()
    }

    companion object {
        const val MAX_RETRY_COUNT = 3
        const val DEFAULT_BSN = "999991772"
        const val DEFAULT_PIN = "122222"
        const val DEFAULT_RECOVERY_CODE = "54aa94af2afc4da286967253a33a61410f0d069c0d77ff748fd83e9fc82c7526"
    }
}
