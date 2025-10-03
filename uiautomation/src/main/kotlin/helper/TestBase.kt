package helper

import com.codeborne.selenide.Selenide
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.extension.ExtendWith
import util.TestInfoHandler.Companion.processTestInfo

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
        try {
            Selenide.closeWindow()
        } catch (e: Exception) {
            // Ignore
        }
        Selenide.closeWebDriver()
    }

    companion object {
        const val MAX_RETRY_COUNT = 3
        const val DEFAULT_BSN = "999991772"
        const val DEFAULT_PIN = "122222"
    }
}
