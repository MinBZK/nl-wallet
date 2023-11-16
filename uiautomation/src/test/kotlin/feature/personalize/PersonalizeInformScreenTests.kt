package feature.personalize

import helper.TestBase
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Test
import screen.digid.DigidApplyWebPage
import screen.digid.DigidLoginStartWebPage
import screen.introduction.IntroductionConditionsScreen
import screen.introduction.IntroductionExpectationsScreen
import screen.introduction.IntroductionPrivacyScreen
import screen.introduction.IntroductionScreen
import screen.personalize.PersonalizeInformScreen
import screen.personalize.PersonalizeNoDigidScreen
import screen.security.PinScreen
import screen.security.SetupSecurityCompletedScreen

@DisplayName("UC 3.1 - App informs User before personalization [PVW-1034]")
class PersonalizeInformScreenTests : TestBase() {

    private val chosenPin = "122222"

    private lateinit var personalizeInformScreen: PersonalizeInformScreen

    @BeforeEach
    fun setUp() {
        val introductionScreen = IntroductionScreen()
        val expectationsScreen = IntroductionExpectationsScreen()
        val privacyScreen = IntroductionPrivacyScreen()
        val conditionsScreen = IntroductionConditionsScreen()
        val pinScreen = PinScreen()
        val setupSecurityCompletedScreen = SetupSecurityCompletedScreen()

        // Start all tests on personalize inform screen
        introductionScreen.clickSkipButton()
        expectationsScreen.clickNextButton()
        privacyScreen.clickNextButton()
        conditionsScreen.clickNextButton()
        pinScreen.enterPin(chosenPin)
        pinScreen.enterPin(chosenPin)
        setupSecurityCompletedScreen.clickNextButton()

        personalizeInformScreen = PersonalizeInformScreen()
    }

    @Test
    @DisplayName("1. The app first displays a screen to the user explaining that to personalize their wallet they must log in with the DigiD App that needs to be activated at level High.")
    fun verifyPersonalizeInformScreen() {
        assertTrue(personalizeInformScreen.visible(), "personalize inform screen is not visible")
    }

    @Test
    @DisplayName("2. The app offers guidance for when the user has no DigiD: the user is referred to the DigiD website.")
    fun verifyNoDigidWebsiteRedirect() {
        personalizeInformScreen.clickNoDigidButton()

        val personalizeNoDigidScreen = PersonalizeNoDigidScreen()
        assertTrue(personalizeNoDigidScreen.visible(), "personalize no digid screen is not visible")

        personalizeNoDigidScreen.clickApplyForDigidButton()
        personalizeNoDigidScreen.switchToWebView()

        val digidApplyWebPage = DigidApplyWebPage()
        assertTrue(digidApplyWebPage.visible(), "digid apply web page is not visible")
    }

    @Test
    @DisplayName("3. The app offers a button for the user to Log in with DigiD.")
    fun verifyLoginDigidButton() {
        personalizeInformScreen.loginWithDigidButtonVisible()
    }

    @Test
    @DisplayName("4. When the user hits 'Log in with DigiD', DigiD authentication starts.")
    fun verifyLoginDigidRedirect() {
        personalizeInformScreen.clickLoginWithDigidButton()
        personalizeInformScreen.switchToWebView()

        val digidLoginStartWebPage = DigidLoginStartWebPage()
        assertTrue(digidLoginStartWebPage.visible(), "digid login start web page is not visible")
    }
}
