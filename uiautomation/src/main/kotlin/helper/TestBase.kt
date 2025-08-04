package helper

import com.codeborne.selenide.Selenide
import data.TestConfigRepository.Companion.testConfig
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.extension.ExtendWith
import util.TestInfoHandler.Companion.processTestInfo

@ExtendWith(TestResultsListener::class)
@ExtendWith(ServiceHelper::class)
open class TestBase {

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

        // Close web driver (remote closes in TestWatcher)
        if (!testConfig.remote) {
            Selenide.closeWebDriver()
        }
    }

    companion object {
        const val MAX_RETRY_COUNT = 3
        const val DEFAULT_BSN = "999991772"
    }
}
