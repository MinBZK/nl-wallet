package feature.dashboard

import helper.GbaDataHelper
import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertAll
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
    private lateinit var gbaData: GbaDataHelper

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.Dashboard)

        dashboardScreen = DashboardScreen()
        gbaData = GbaDataHelper()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1 The card overview page displays all cards currently available in the app. 2. Each card is recognizable as a physical card (fixed ratio, unless the font size is too big, then the card ratio constraint is relaxed) and includes the following: a title, subtitle, background image, logo, CTA button.[${JIRA_ID}]")
    @Tags(Tag("smoke"))
    fun verifyIssuedCardsVisible(testInfo: TestInfo) {
        setUp(testInfo)
        assertAll(
            { assertTrue(dashboardScreen.pidCardsVisible(), "Expected cards are not visible") },
            { assertTrue(dashboardScreen.cardTitlesVisible(), "card title are not visible") },
            { assertTrue(dashboardScreen.cardButtonsVisible(), "card buttons are not visible") },
            { assertTrue(dashboardScreen.cardSubtitleVisible(gbaData.getValueByField(GbaDataHelper.Field.FIRST_NAME,"999991772")), "pid card subtitle is not visible") },
            { assertTrue(dashboardScreen.cardSubtitleVisible(gbaData.getValueByField(GbaDataHelper.Field.CITY,"999991772")), "adress card subtitle is not visible") },
        )
    }

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
