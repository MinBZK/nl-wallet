package uiTests

import com.codeborne.selenide.Configuration
import com.codeborne.selenide.Selenide
import com.codeborne.selenide.WebDriverRunner.getWebDriver
import com.codeborne.selenide.logevents.SelenideLogger
import config.TestDataConfig.RemoteOrLocal
import config.TestDataConfig.Companion.testDataConfig
import drivers.BrowserstackMobileDriver
import drivers.LocalMobileDriver
import helper.Attach
import io.qameta.allure.Allure
import io.qameta.allure.Allure.ThrowableRunnableVoid
import io.qameta.allure.selenide.AllureSelenide
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeAll
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.TestInfo

import server.AppiumServiceProvider
import util.setupTestTagHandler.Companion.handleTestTags
import java.time.Duration

open class TestBase {

    @BeforeEach
    fun startDriver(testInfo: TestInfo) {
        handleTestTags(testInfo)
        SelenideLogger.addListener("AllureSelenide", AllureSelenide())
        Selenide.open()
        getWebDriver().manage().timeouts().implicitlyWait(Duration.ofSeconds(10))
    }

    @AfterEach
    fun afterEach() {
        val sessionId: String = Attach.sessionId()
        Attach.screenshotWithTimeStamp()
        Allure.step("Close driver", ThrowableRunnableVoid {
            Selenide.closeWebDriver()
            AppiumServiceProvider.stopServer()
        })
        if (testDataConfig.remoteOrLocal == RemoteOrLocal.remote) {
            Attach.video(sessionId)
        }
    }

    companion object {
        @JvmStatic
        @BeforeAll
        fun setup() {
            if (testDataConfig.remoteOrLocal == RemoteOrLocal.remote) {
                Configuration.browser = BrowserstackMobileDriver::class.java.name
            } else {
                Configuration.browser = LocalMobileDriver::class.java.name
            }
            Configuration.browserSize = null
        }
    }
}