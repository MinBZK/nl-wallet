package feature.card

import helper.TestBase
import navigator.CardNavigator
import navigator.CardScreen
import navigator.OnboardingNavigator
import navigator.OnboardingScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junitpioneer.jupiter.RetryingTest
import screen.card.CardDataScreen
import screen.card.CardDetailScreen
import screen.card.CardHistoryScreen
import screen.dashboard.DashboardScreen

@DisplayName("UC 7.2 - App shows card detail overview [PVW-1228]")
class CardDetailTests : TestBase() {

    private lateinit var cardDetailScreen: CardDetailScreen

    @BeforeEach
    fun setUp() {
        OnboardingNavigator().toScreen(OnboardingScreen.Dashboard)
        CardNavigator().toScreen(CardScreen.CardDetail)

        cardDetailScreen = CardDetailScreen()
    }

    @RetryingTest(MAX_RETRY_COUNT, name = "{displayName}")
    @DisplayName("1. The Card detail page shows the actual card data as stored in the app.")
    fun verifyCardDetailScreen() {
        assertTrue(cardDetailScreen.visible(), "card detail screen is not visible")
    }

    @RetryingTest(MAX_RETRY_COUNT, name = "{displayName}")
    @DisplayName("2. The Card detail page shows the Card face (exactly the same as on the dashboard, minus the 'show details' button).")
    fun verifyCardDetailButtonAbsent() {
        assertTrue(cardDetailScreen.cardFaceElements(), "card face for detail screen is not visible and/or correct")
    }

    @RetryingTest(MAX_RETRY_COUNT, name = "{displayName}")
    @DisplayName("3. The Card detail page shows: issuer name, empty history state")
    fun verifyDataAndHistoryState() {
        assertTrue(cardDetailScreen.dataAndHistoryStates(), "data and/or history state not not visible and/or correct")
    }

    @RetryingTest(MAX_RETRY_COUNT, name = "{displayName}")
    @DisplayName("4. The Card detail page offers a button to reveal the card attributes.")
    fun verifyCardDataButton() {
        cardDetailScreen.clickCardDataButton()

        val cardDataScreen = CardDataScreen()
        assertTrue(cardDataScreen.visible(), "card data screen is not visible")
    }

    @RetryingTest(MAX_RETRY_COUNT, name = "{displayName}")
    @DisplayName("5. The Card detail page offers a button to display card history.")
    fun verifyCardHistoryButton() {
        cardDetailScreen.clickCardHistoryButton()

        val cardHistoryScreen = CardHistoryScreen()
        assertTrue(cardHistoryScreen.visible(), "card history screen is not visible")
    }

    @RetryingTest(MAX_RETRY_COUNT, name = "{displayName}")
    @DisplayName("6. The Card detail page offers a button to go back to the card overview.")
    fun verifyBackButton() {
        cardDetailScreen.clickBottomBackButton()

        val dashboardScreen = DashboardScreen()
        assertTrue(dashboardScreen.visible(), "dashboard screen is not visible")
    }
}
