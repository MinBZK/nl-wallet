package feature.history

import helper.LocalizationHelper
import helper.LocalizationHelper.Translation.PID_CARD_TITLE
import helper.TestBase
import navigator.MenuNavigator
import navigator.OnboardingNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertAll
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.dashboard.DashboardScreen
import screen.disclosure.DisclosureApproveOrganizationScreen
import screen.history.HistoryDetailScreen
import screen.history.HistoryOverviewScreen
import screen.menu.MenuScreen
import screen.security.PinScreen
import screen.web.rp.RelyingPartyAmsterdamWebPage
import screen.web.rp.RelyingPartyOverviewWebPage

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${HistoryOverviewTests.USE_CASE} App logs PID/address issuance and disclosure events [${HistoryOverviewTests.JIRA_ID}]")
class HistoryOverviewTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 3.1"
        const val JIRA_ID = "PVW-1038"
    }

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()

        val overviewWebPage = RelyingPartyOverviewWebPage()
        val platform = overviewWebPage.platformName()

        overviewWebPage.clickAmsterdamButton()
        val amsterdamWebPage = RelyingPartyAmsterdamWebPage()
        amsterdamWebPage.openSameDeviceWalletFlow(platform)
        amsterdamWebPage.switchToAppContext()

        val disclosureScreen = DisclosureApproveOrganizationScreen()
        disclosureScreen.login()
        PinScreen().enterPin(OnboardingNavigator.PIN)

        disclosureScreen.closeDialog()
        DashboardScreen().clickMenuButton()
        MenuScreen().clickHistoryButton()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.A log entries are added for the PID issuance and disclosure events. [${JIRA_ID}]")
    fun verifyIssuanceHistoryEntries(testInfo: TestInfo) {
        setUp(testInfo)

        val historyOverviewScreen = HistoryOverviewScreen()
        assertAll(
            { assertTrue(historyOverviewScreen.visible(), "history overview screen is not visible") },
            { assertTrue(historyOverviewScreen.pidIssuanceLogEntryVisible(), "pid issuance log entry is not visible") },
            { assertTrue(historyOverviewScreen.addressIssuanceLogEntryVisible(), "address issuance log entry is not visible") },
            { assertTrue(historyOverviewScreen.loginDisclosureLogEntryVisible(), "login log entry is not visible") }
        )

        historyOverviewScreen.clickPidCardTitle()

        val historyDetailScreen = HistoryDetailScreen()
        val i18n = LocalizationHelper()
        assertAll(
            { assertTrue(historyDetailScreen.issuanceOrganizationVisible("RvIG"), "organization not visible") },
            { assertTrue(historyDetailScreen.titleCorrectForIssuance(i18n.translate(PID_CARD_TITLE)), "title not visible") }
        )

        historyDetailScreen.clickBottomBackButton()

        historyOverviewScreen.clickLoginEntryTitle()
        assertAll(
            { assertTrue(historyDetailScreen.disclosureOrganizationVisible("Gemeente Amsterdam"), "organization not visible") },
            { assertTrue(historyDetailScreen.titleCorrectForLogin("Gemeente Amsterdam"), "title not visible") }
        )
    }

    /**
     * 2. A log entry is only stored locally on the smartphone.
     * >> This requirement hard, if not impossible to be tested in an e2e setup and should be validated during an audit of the app.
     */
}
