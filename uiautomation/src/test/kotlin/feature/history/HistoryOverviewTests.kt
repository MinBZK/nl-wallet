package feature.history

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.dashboard.DashboardScreen
import screen.history.HistoryOverviewScreen
import screen.menu.MenuScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${HistoryOverviewTests.USE_CASE} App logs PID/address issuance [${HistoryOverviewTests.JIRA_ID}]")
class HistoryOverviewTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 3.1"
        const val JIRA_ID = "PVW-1038"
    }

    private lateinit var historyOverviewScreen: HistoryOverviewScreen

    @BeforeEach
    fun setUp() {
        OnboardingNavigator().toScreen(OnboardingScreen.Dashboard)
        DashboardScreen().clickMenuButton()
        MenuScreen().clickHistoryButton()

        historyOverviewScreen = HistoryOverviewScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.A log entry is added for the PID issuance event. [${JIRA_ID}]")
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
    }

    /**
     * 2. A log entry is only stored locally on the smartphone.
     * >> This requirement hard, if not impossible to be tested in an e2e setup and should be validated during an audit of the app.
     */
}
