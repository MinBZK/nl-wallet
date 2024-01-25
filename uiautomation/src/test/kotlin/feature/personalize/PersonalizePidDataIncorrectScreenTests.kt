package feature.personalize

import helper.TestBase
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Test
import screen.digid.DigidLoginMockWebPage
import screen.digid.DigidLoginStartWebPage
import screen.introduction.IntroductionConditionsScreen
import screen.introduction.IntroductionExpectationsScreen
import screen.introduction.IntroductionPrivacyScreen
import screen.introduction.IntroductionScreen
import screen.personalize.PersonalizeInformScreen
import screen.personalize.PersonalizePidDataIncorrectScreen
import screen.personalize.PersonalizePidPreviewScreen
import screen.security.PinScreen
import screen.security.SetupSecurityCompletedScreen

@DisplayName("UC 3.1 - User reports that PID/address is incorrect [PVW-1040]")
class PersonalizePidDataIncorrectScreenTests : TestBase() {

    private val chosenPin = "122222"

    private lateinit var personalizePidDataIncorrectScreen: PersonalizePidDataIncorrectScreen

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

        // Start all tests on pid preview screen
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
        personalizePidPreviewScreen.clickRejectButton()

        personalizePidDataIncorrectScreen = PersonalizePidDataIncorrectScreen()
    }

    @Test
    @DisplayName("1. When the User rejects, the App shows the 'Incorrect data support screen' that informs the User about what to do in case the data are not correct.")
    fun verifyPersonalizePidDataIncorrectScreen() {
        assertTrue(personalizePidDataIncorrectScreen.visible(), "personalize pid data incorrect screen is not visible")
    }

    @Test
    @DisplayName("2. The App offers a button for the user to go back to the process.")
    fun verifyBackButton() {
        personalizePidDataIncorrectScreen.clickBottomBackButton()

        val personalizePidPreviewScreen = PersonalizePidPreviewScreen()
        assertTrue(personalizePidPreviewScreen.visible(), "personalize pid preview screen is not visible")
    }

    @Test
    @DisplayName("3. The App offers a button to not add the data, which returns to the 'login with DigiD screen'.")
    fun verifyRejectPidPreviewButton() {
        personalizePidDataIncorrectScreen.clickBottomPrimaryButton()

        val personalizeInformScreen = PersonalizeInformScreen()
        assertTrue(personalizeInformScreen.visible(), "personalize inform screen is not visible")
    }
}