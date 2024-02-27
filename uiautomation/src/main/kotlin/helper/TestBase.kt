package helper

import com.codeborne.selenide.Configuration
import com.codeborne.selenide.Selenide
import com.codeborne.selenide.WebDriverRunner.getWebDriver
import com.codeborne.selenide.logevents.SelenideLogger
import config.RemoteOrLocal
import config.TestDataConfig.Companion.testDataConfig
import driver.BrowserStackMobileDriver
import driver.LocalMobileDriver
import io.appium.java_client.android.AndroidDriver
import io.qameta.allure.Allure
import io.qameta.allure.Allure.ThrowableRunnableVoid
import io.qameta.allure.selenide.AllureSelenide
import org.junit.jupiter.api.AfterAll
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeAll
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.extension.ExtendWith
import org.openqa.selenium.remote.RemoteWebDriver
import service.AppiumServiceProvider
import util.SetupTestTagHandler.Companion.handleTestTags

@ExtendWith(TestResultsListener::class)
open class TestBase {

    @BeforeEach
    fun startDriver(testInfo: TestInfo) {
        handleTestTags(testInfo)
        sessionName = testInfo.displayName

        SelenideLogger.addListener("AllureSelenide", AllureSelenide())

        // Start driver
        Selenide.open()
    }

    @AfterEach
    fun afterEach() {
        try {
            // Close browser tab
            Selenide.closeWindow()
        } catch (e: Exception) {
            // Ignore
        }

        if (testDataConfig.remoteOrLocal == RemoteOrLocal.Local) {
            Allure.step("Close driver", ThrowableRunnableVoid {
                Selenide.closeWebDriver()
            })
        }
    }

    protected fun restartApp() {
        val driver = getWebDriver() as RemoteWebDriver
        val platform = driver.capabilities.platformName.name
        val packageName = testDataConfig.appPackage
        if (platform == "ANDROID") {
            val androidDriver = driver as AndroidDriver
            androidDriver.terminateApp(packageName)
            androidDriver.activateApp(packageName)
        } else {
            throw Exception("Platform $platform is not supported")
        }
    }

    companion object {
        const val MAX_RETRY_COUNT = 3

        var sessionName: String = ""

        @JvmStatic
        @BeforeAll
        fun setup() {
            // Start Appium service if running locally
            if (testDataConfig.remoteOrLocal == RemoteOrLocal.Local) {
                AppiumServiceProvider.startService()
            }

            when (testDataConfig.remoteOrLocal) {
                RemoteOrLocal.Local -> Configuration.browser = LocalMobileDriver::class.java.name
                RemoteOrLocal.Remote -> Configuration.browser = BrowserStackMobileDriver::class.java.name
            }
            Configuration.browserSize = null
        }

        @JvmStatic
        @AfterAll
        fun destroy() {
            // Stop Appium service if running locally
            if (testDataConfig.remoteOrLocal == RemoteOrLocal.Local) {
                AppiumServiceProvider.stopService()
            }
        }
    }
}
