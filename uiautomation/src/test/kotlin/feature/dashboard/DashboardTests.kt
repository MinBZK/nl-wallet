package feature.dashboard

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.card.CardDetailScreen
import screen.dashboard.DashboardScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${DashboardTests.USE_CASE} App shows all cards available in the app [${DashboardTests.JIRA_ID}]")
class DashboardTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 7.1"
        const val JIRA_ID = "PVW-1227"
    }

    private lateinit var dashboardScreen: DashboardScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.Dashboard)

        dashboardScreen = DashboardScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1 The card overview page displays all cards currently available in the app. [${JIRA_ID}]")
    @Tags(Tag("smoke"))
    fun verifyIssuedCardsVisible(testInfo: TestInfo) {
        setUp(testInfo)
        assertTrue(dashboardScreen.cardsVisible(), "Expected cards are not visible")
    }

    /**
     * 2. Each card is recognizable as a physical card (fixed ratio, unless the font size is too big, then the card ratio constraint is relaxed) and includes the following: a title, subtitle, background image, logo, CTA button.
     * >> Manual test: https://SSSS/jira/browse/PVW-1976
     */

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.3 The card information (and images) is displayed in the active language. [${JIRA_ID}]")
    @Tags(Tag("english"))
    fun verifyActiveLanguage(testInfo: TestInfo) {
        setUp(testInfo)
        assertTrue(dashboardScreen.cardFaceTextsInActiveLanguage(), "Card face texts are not in active language")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.4 Tapping the card opens the card's details. [${JIRA_ID}]")
    fun verifyCardDetailScreen(testInfo: TestInfo) {
        setUp(testInfo)
        dashboardScreen.clickPidCard()

        val cardDetailScreen = CardDetailScreen()
        assertTrue(cardDetailScreen.visible(), "card detail screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.5 The card sorting is initially fixed: PID is first, Address is second. [${JIRA_ID}]")
    fun verifyCardsFixedSorting(testInfo: TestInfo) {
        setUp(testInfo)
        assertTrue(dashboardScreen.checkCardSorting(), "card sorting not as expected")
    }
}
