package feature.menu_and_settings

import helper.TestBase
import navigator.MenuNavigator
import navigator.screen.MenuNavigatorScreen
import screen.introduction.IntroductionScreen
import screen.menu.MenuScreen
import screen.settings.ClearDataDialog
import screen.settings.SettingsScreen
import org.junit.jupiter.api.Assertions.assertAll
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("UC9.4 Wipe all app data")
class ClearDataTests : TestBase() {

    private lateinit var clearDataDialog: ClearDataDialog

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickSettingsButton()
        SettingsScreen().clickClearDataButton()

        clearDataDialog = ClearDataDialog()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC28 Delete App data")
    fun verifyClearData(testInfo: TestInfo) {
        setUp(testInfo)
        assertAll(
            { assertTrue(clearDataDialog.informVisible(), "consequence inform is not visible") },
            { assertTrue(clearDataDialog.cancelButtonVisible(), "cancel button is not visible") },
            { assertTrue(clearDataDialog.confirmButtonVisible(), "confirm button is not visible") }
        )

        clearDataDialog.clickConfirmButton()
        assertTrue(IntroductionScreen().page1Visible(), "introduction screen is not visible")
    }
}
