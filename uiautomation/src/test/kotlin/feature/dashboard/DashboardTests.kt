package feature.dashboard

import helper.TestBase
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.Test
import screen.card.CardDetailScreen
import screen.dashboard.DashboardScreen
import screen.digid.DigidLoginMockWebPage
import screen.digid.DigidLoginStartWebPage
import screen.introduction.IntroductionConditionsScreen
import screen.introduction.IntroductionExpectationsScreen
import screen.introduction.IntroductionPrivacyScreen
import screen.introduction.IntroductionScreen
import screen.personalize.PersonalizeInformScreen
import screen.personalize.PersonalizePidPreviewScreen
import screen.personalize.PersonalizeSuccessScreen
import screen.security.PinScreen
import screen.security.SetupSecurityCompletedScreen

@DisplayName("UC 7.1 - App shows all cards available in the app [PVW-1227]")
class DashboardTests : TestBase() {

    private val chosenPin = "122222"

    private lateinit var dashboardScreen: DashboardScreen

    @BeforeEach
    fun setUp() {
        val introductionScreen = IntroductionScreen()
        val expectationsScreen = IntroductionExpectationsScreen()
        val privacyScreen = IntroductionPrivacyScreen()
        val conditionsScreen = IntroductionConditionsScreen()
        val pinScreen = PinScreen()
        val setupSecurityCompletedScreen = SetupSecurityCompletedScreen()
        val personalizeInformScreen = PersonalizeInformScreen()
        val digidLoginStartWebPage = DigidLoginStartWebPage()
        val digidLoginMockWebPage = DigidLoginMockWebPage()
        val personalizePidPreviewScreen = PersonalizePidPreviewScreen()
        val personalizeSuccessScreen = PersonalizeSuccessScreen()

        // Start all tests on dashboard screen
        introductionScreen.clickSkipButton()
        expectationsScreen.clickNextButton()
        privacyScreen.clickNextButton()
        conditionsScreen.clickNextButton()
        pinScreen.enterPin(chosenPin)
        pinScreen.enterPin(chosenPin)
        setupSecurityCompletedScreen.clickNextButton()
        personalizeInformScreen.clickLoginWithDigidButton()
        personalizeInformScreen.switchToWebView()
        digidLoginStartWebPage.clickMockLoginButton()
        digidLoginMockWebPage.clickLoginButton()
        personalizePidPreviewScreen.switchToApp()
        personalizePidPreviewScreen.clickAcceptButton()
        pinScreen.enterPin(chosenPin)
        personalizeSuccessScreen.clickNextButton()

        dashboardScreen = DashboardScreen()
    }

    @Test
    @DisplayName("1. The card overview page displays all cards currently available in the app.")
    fun verifyIssuedCardsVisible() {
        assertTrue(dashboardScreen.cardsVisible(), "Expected cards are not visible")
    }

    //@Test
    @DisplayName("2. Each card is recognizable as a physical card (fixed ratio, unless the font size is too big, then the card ratio constraint is relaxed) and includes the following: a title, subtitle, background image, logo, CTA button.")
    fun verifyCardPhysicalFixedRatioAndFaceElements() {
        // Manual test: https://SSSS/jira/browse/PVW-1976
    }

    @Test
    @DisplayName("3. The card information (and images) is displayed in the active language.")
    @Tags(Tag("english"))
    fun verifyActiveLanguage() {
        assertTrue(dashboardScreen.cardFaceTextsInActiveLanguage(), "Card face texts are not in active language")
    }

    @Test
    @DisplayName("4. Tapping the card opens the card's details [UC 7.2]")
    fun verifyCardDetailScreen() {
        dashboardScreen.clickPidCard()

        val cardDetailScreen = CardDetailScreen()
        assertTrue(cardDetailScreen.visible(), "card detail screen is not visible")
    }

    @Test
    @DisplayName("5. The card sorting is initially fixed: PID is first, Address is second.")
    fun verifyCardsFixedSorting() {
        assertTrue(dashboardScreen.checkCardSorting(), "card sorting not as expected")
    }
}
