package helper

import com.codeborne.selenide.Selenide
import com.codeborne.selenide.WebDriverRunner
import data.TwoDeviceConfigRepository.Companion.twoDeviceConfig
import driver.LocalMobileDriver
import io.appium.java_client.AppiumDriver
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.TestInfo
import service.AppiumServiceProvider
import util.TestInfoHandler.Companion.processTestInfo

open class TwoDeviceTestBase {

    protected lateinit var sourceDriver: AppiumDriver
    protected lateinit var targetDriver: AppiumDriver

    fun startDrivers(testInfo: TestInfo) {
        processTestInfo(testInfo)
        AppiumServiceProvider.startService(sessionOverride = false)
        sourceDriver = LocalMobileDriver.createDriver(twoDeviceConfig.source, index = 1)
        targetDriver = LocalMobileDriver.createDriver(twoDeviceConfig.destination, index = 2)
    }

    fun useSourceDevice(block: () -> Unit) {
        WebDriverRunner.setWebDriver(sourceDriver)
        block()
    }

    fun useTargetDevice(block: () -> Unit) {
        WebDriverRunner.setWebDriver(targetDriver)
        block()
    }

    // Drivers created with Selenide's WebDriverRunner.setWebDriver() can/should be closed with Selenide.closeWebDriver()
    @AfterEach
    fun closeDrivers() {
        if (::sourceDriver.isInitialized) {
            clearBrowser(sourceDriver)
            WebDriverRunner.setWebDriver(sourceDriver)
            Selenide.closeWebDriver()
        }
        if (::targetDriver.isInitialized) {
            clearBrowser(targetDriver)
            WebDriverRunner.setWebDriver(targetDriver)
            Selenide.closeWebDriver()
        }
    }

    companion object {
        const val MAX_RETRY_COUNT = TestBase.MAX_RETRY_COUNT
        const val DEFAULT_BSN = TestBase.DEFAULT_BSN
        const val DEFAULT_PIN = TestBase.DEFAULT_PIN
    }
}
