package feature.cards_and_history

import helper.GbaDataHelper
import helper.TasDataHelper
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
@DisplayName("Use Case 7.1 Show all available cards")
class DashboardTests : TestBase() {

    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var cardDetailScreen: CardDetailScreen
    private lateinit var gbaData: GbaDataHelper
    private lateinit var tasData: TasDataHelper

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.Dashboard)

        dashboardScreen = DashboardScreen()
        cardDetailScreen = CardDetailScreen()
        gbaData = GbaDataHelper()
        tasData = TasDataHelper()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC32 Show all available cards")
    @Tags(Tag("smoke"), Tag("a11yBatch1"))
    fun verifyIssuedCardsVisible(testInfo: TestInfo) {
        setUp(testInfo)
        assertAll(
            { assertTrue(dashboardScreen.cardTitlesVisible(), "card title are not visible") },
            { assertTrue(dashboardScreen.cardButtonsVisible(), "card buttons are not visible") },
            { assertTrue(dashboardScreen.cardSubtitleVisible(gbaData.getValueByField(GbaDataHelper.Field.FIRST_NAME, DEFAULT_BSN)), "pid card subtitle is not visible") },
            { assertTrue(dashboardScreen.cardSubtitleVisible(gbaData.getValueByField(GbaDataHelper.Field.CITY, DEFAULT_BSN)), "adress card subtitle is not visible") },
            { assertTrue(dashboardScreen.checkCardSorting(), "card sorting not as expected") },
        )

        dashboardScreen.clickCard(tasData.getPidDisplayName())
        assertTrue(cardDetailScreen.visible(), "card detail screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC32 The card information (and images) is displayed in the active language.")
    @Tags(Tag("english"))
    fun verifyActiveLanguage(testInfo: TestInfo) {
        setUp(testInfo)
        assertTrue(dashboardScreen.cardFaceTextsInActiveLanguage(), "Card face texts are not in active language")
    }
}
