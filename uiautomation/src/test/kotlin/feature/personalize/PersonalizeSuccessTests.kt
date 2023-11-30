package feature.personalize

import helper.TestBase
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Test
import screen.card.CardOverviewScreen
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

@DisplayName("UC 3.1 - App confirms PID issuance to user [PVW-1039]")
class PersonalizeSuccessTests : TestBase() {

    private val chosenPin = "122222"

    private lateinit var personalizeSuccessScreen: PersonalizeSuccessScreen

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

        // Start all tests on digid login start web page
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

        personalizeSuccessScreen = PersonalizeSuccessScreen()
    }

    @Test
    @DisplayName("1. When PID was issued successfully, the App displays a confirmation to the User.")
    fun verifyPersonalizeSuccessScreen() {
        assertTrue(personalizeSuccessScreen.visible(), "personalize loading screen is not visible")
    }

    //@Test
    @DisplayName("2. The confirmation includes a success message.")
    fun verifySuccessMessage() {
        // Manual test: https://SSSS/jira/browse/PVW-1771
    }

    //@Test
    @DisplayName("3. The confirmation includes the issued cards (PID + Address): card, title.")
    fun verifyIssuedCards() {
        // Manual test: https://SSSS/jira/browse/PVW-1772
    }

    @Test
    @DisplayName("4. The App offers an entrance to enter the wallet which brings the User to the Dashboard.")
    fun verifyNavigateToDashboardButton() {
        personalizeSuccessScreen.clickNextButton()

        val cardOverviewScreen = CardOverviewScreen()
        assertTrue(cardOverviewScreen.visible(), "card overview screen is not visible")
    }
}
