package uiTests

import com.codeborne.selenide.Configuration
import com.codeborne.selenide.Selenide
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
import server.AppiumServiceProvider

open class TestBase {

    @BeforeEach
    fun startDriver() {
        SelenideLogger.addListener("AllureSelenide", AllureSelenide())
        Selenide.open()
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