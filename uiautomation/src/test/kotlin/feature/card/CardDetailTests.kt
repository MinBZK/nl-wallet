package feature.card

import helper.TestBase
import navigator.CardNavigator
import navigator.screen.CardNavigatorScreen
import org.junit.jupiter.api.Assertions.assertAll
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.card.CardDataScreen
import screen.card.CardDetailScreen
import screen.card.CardHistoryScreen
import screen.dashboard.DashboardScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${CardDetailTests.USE_CASE} App shows card detail overview [${CardDetailTests.JIRA_ID}]")
class CardDetailTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 7.2"
        const val JIRA_ID = "PVW-1228"
    }

    private lateinit var cardDetailScreen: CardDetailScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        CardNavigator().toScreen(CardNavigatorScreen.CardDetail)

        cardDetailScreen = CardDetailScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1 The Card detail page shows the actual card data as stored in the app. 2 The Card detail page shows the Card face (exactly the same as on the dashboard, minus the 'show details' button). 3 The Card detail page shows: issuer name, empty history state.  [${JIRA_ID}]")
    fun verifyCardDetailScreen(testInfo: TestInfo) {
        setUp(testInfo)
        assertAll(
            { assertTrue(cardDetailScreen.visible(), "card detail screen is not visible") },
            { assertTrue(cardDetailScreen.cardFaceElements(), "card face for detail screen is not visible and/or correct") },
            { assertTrue(cardDetailScreen.issuerAndHistoryStates(), "issuer and/or history state not not visible and/or correct") }
        )
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.4 The Card detail page offers a button to reveal the card attributes. [${JIRA_ID}]")
    fun verifyCardDataButton(testInfo: TestInfo) {
        setUp(testInfo)
        cardDetailScreen.clickCardDataButton()

        val cardDataScreen = CardDataScreen()
        assertTrue(cardDataScreen.visible(), "card data screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.5 The Card detail page offers a button to display card history. [${JIRA_ID}]")
    fun verifyCardHistoryButton(testInfo: TestInfo) {
        setUp(testInfo)
        cardDetailScreen.clickCardHistoryButton()

        val cardHistoryScreen = CardHistoryScreen()
        assertTrue(cardHistoryScreen.visible(), "card history screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.6 The Card detail page offers a button to go back to the card overview. [${JIRA_ID}]")
    fun verifyBackButton(testInfo: TestInfo) {
        setUp(testInfo)
        cardDetailScreen.clickBottomBackButton()

        val dashboardScreen = DashboardScreen()
        assertTrue(dashboardScreen.visible(), "dashboard screen is not visible")
    }
}
