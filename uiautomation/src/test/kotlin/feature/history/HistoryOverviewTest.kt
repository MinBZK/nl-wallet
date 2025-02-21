package feature.history

import helper.TestBase
import navigator.MenuNavigator
import navigator.OnboardingNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.disclosure.DisclosureApproveOrganizationScreen
import screen.history.HistoryDetailScreen
import screen.history.HistoryOverviewScreen
import screen.menu.MenuScreen
import screen.security.PinScreen
import screen.web.rp.RelyingPartyAmsterdamWebPage
import screen.web.rp.RelyingPartyOverviewWebPage

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${HistoryOverviewTest.USE_CASE} App logs PID/address issuance and disclosure events [${HistoryOverviewTest.JIRA_ID}]")
class HistoryOverviewTest : TestBase() {

    companion object {
        const val USE_CASE = "UC 3.1"
        const val JIRA_ID = "PVW-1038"
    }

    private lateinit var historyOverviewScreen: HistoryOverviewScreen
    private lateinit var overviewWebPage: RelyingPartyOverviewWebPage
    private lateinit var amsterdamWebPage: RelyingPartyAmsterdamWebPage
    private lateinit var disclosureScreen: DisclosureApproveOrganizationScreen
    private lateinit var pinScreen: PinScreen
    private lateinit var historyDetailScreen: HistoryDetailScreen

    @BeforeEach
    fun setUp() {
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        overviewWebPage = RelyingPartyOverviewWebPage()
        amsterdamWebPage = RelyingPartyAmsterdamWebPage()
        disclosureScreen = DisclosureApproveOrganizationScreen()
        historyDetailScreen = HistoryDetailScreen()
        pinScreen = PinScreen()
        overviewWebPage.clickAmsterdamButton()
        val platform = overviewWebPage.platformName()
        amsterdamWebPage.openSameDeviceWalletFlow(platform)
        amsterdamWebPage.switchToAppContext()
        disclosureScreen.login()
        pinScreen.enterPin(OnboardingNavigator.PIN)
        disclosureScreen.viewActivities()
        historyOverviewScreen = HistoryOverviewScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.A log entries are added for the PID issuance and disclosure events. [${JIRA_ID}]")
    fun verifyIssuanceHistoryEntries() {
        assertTrue(historyOverviewScreen.visible(), "history overview screen is not visible")
        assertTrue(
            historyOverviewScreen.pidIssuanceLogEntryVisible(),
            "pid issuance log entry is not visible"
        )
        assertTrue(
            historyOverviewScreen.addressIssuanceLogEntryVisible(),
            "address issuance log entry is not visible"
        )
        assertTrue(
            historyOverviewScreen.loginDisclosureLogEntryVisible(),
            "login log entry is not visible"
        )
        historyOverviewScreen.clickPidCardTitle()
        assertTrue(historyDetailScreen.issuanceOrganizationVisible("RvIG"), "organization not visible")
        assertTrue(historyDetailScreen.titleCorrectForIssuance("Persoons\u00ADgegevens"), "title not visible")
        historyDetailScreen.clickBottomBackButton()
        historyOverviewScreen.clickLoginEntryTitle()
        assertTrue(historyDetailScreen.disclosureOrganizationVisible("Gemeente Amsterdam"), "organization not visible")
        assertTrue(historyDetailScreen.titleCorrectForLogin("Gemeente Amsterdam"), "title not visible")
    }

    /**
     * 2. A log entry is only stored locally on the smartphone.
     * >> This requirement hard, if not impossible to be tested in an e2e setup and should be validated during an audit of the app.
     */
}
