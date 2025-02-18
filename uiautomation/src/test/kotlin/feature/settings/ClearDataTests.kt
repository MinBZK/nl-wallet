package feature.settings

import helper.TestBase
import navigator.MenuNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Nested
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.introduction.IntroductionScreen
import screen.menu.MenuScreen
import screen.settings.ClearDataDialog
import screen.settings.SettingsScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${ClearDataTests.USE_CASE} Wipe all data from the App [${ChangeLanguageTests.JIRA_ID}]")
class ClearDataTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 9.4"
        const val JIRA_ID = "PVW-2231"
    }

    private lateinit var clearDataDialog: ClearDataDialog

    fun setUp() {
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickSettingsButton()
        SettingsScreen().clickClearDataButton()

        clearDataDialog = ClearDataDialog()
    }

    @Nested
    @DisplayName("$USE_CASE.1 When the User enters this feature from the App Menu or the Forgot PIN screen: [$JIRA_ID]")
    inner class ClearDataInform {

        @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
        @DisplayName("$USE_CASE.1.1 the App informs the User of the consequences of this action. [$JIRA_ID]")
        fun verifyConsequenceInform() {
            setUp()
            assertTrue(clearDataDialog.informVisible(), "consequence inform is not visible")
        }

        @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
        @DisplayName("$USE_CASE.1.2 the App offers the User the option to cancel, aborting this flow. [$JIRA_ID]")
        fun verifyCancelButton() {
            setUp()
            assertTrue(clearDataDialog.cancelButtonVisible(), "cancel button is not visible")
        }

        @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
        @DisplayName("$USE_CASE.1.3 the App offers the User the option to continue, continuing this flow. [$JIRA_ID]")
        fun verifyConfirmButton() {
            setUp()
            assertTrue(clearDataDialog.confirmButtonVisible(), "confirm button is not visible")
        }
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("${ChangeLanguageTests.USE_CASE}.2 If wiping is confirmed, the App and all data stored by the App are completely removed from the device. [${ChangeLanguageTests.JIRA_ID}]")
    fun verifyClearData() {
        setUp()
        clearDataDialog.clickConfirmButton()

        assertTrue(IntroductionScreen().page1Visible(), "introduction screen is not visible")
    }

    /**
     * 3. After wiping, the wallet account still exists at the Wallet Provider (the remaining information is rendered effectively unusable).
     * >> This requirement hard, if not impossible to be tested in an e2e setup and should be validated during an audit of the app.
     */
}
