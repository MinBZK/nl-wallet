package uiTests

import com.codeborne.selenide.Configuration
import com.codeborne.selenide.Selenide
import com.codeborne.selenide.WebDriverRunner.getWebDriver
import com.codeborne.selenide.logevents.SelenideLogger
import config.RemoteOrLocal
import config.TestDataConfig.Companion.testDataConfig
import driver.BrowserstackMobileDriver
import driver.LocalMobileDriver
import helper.Attach
import helper.TestResultsListener
import io.qameta.allure.Allure
import io.qameta.allure.Allure.ThrowableRunnableVoid
import io.qameta.allure.selenide.AllureSelenide
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeAll
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.extension.ExtendWith
import server.AppiumServiceProvider
import util.SetupTestTagHandler.Companion.handleTestTags
import java.time.Duration

@ExtendWith(TestResultsListener::class)
open class TestBase {

    @BeforeEach
    fun startDriver(testInfo: TestInfo) {
        handleTestTags(testInfo)
        sessionName = testInfo.displayName
        SelenideLogger.addListener("AllureSelenide", AllureSelenide())
        Selenide.open()
        getWebDriver().manage().timeouts().implicitlyWait(Duration.ofSeconds(10))
    }

    @AfterEach
    fun afterEach() {
        val sessionId: String = Attach.sessionId()
        Attach.screenshotWithTimeStamp()
        if (testDataConfig.remoteOrLocal == RemoteOrLocal.Remote) {
            Attach.video(sessionId)
        } else {
            Allure.step("Close driver", ThrowableRunnableVoid {
                Selenide.closeWebDriver()
                AppiumServiceProvider.stopServer()
            })
        }
    }

    companion object {
        var sessionName: String = ""

        @JvmStatic
        @BeforeAll
        fun setup() {
            if (testDataConfig.remoteOrLocal == RemoteOrLocal.Remote) {
                Configuration.browser = BrowserstackMobileDriver::class.java.name
            } else {
                Configuration.browser = LocalMobileDriver::class.java.name
            }
            Configuration.browserSize = null
        }
    }
}
