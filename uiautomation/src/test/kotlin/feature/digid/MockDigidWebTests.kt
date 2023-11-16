package feature.digid

import helper.TestBase
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import screen.digid.DigidLoginMockWebPage
import screen.digid.DigidLoginStartWebPage
import screen.introduction.IntroductionConditionsScreen
import screen.introduction.IntroductionExpectationsScreen
import screen.introduction.IntroductionPrivacyScreen
import screen.introduction.IntroductionScreen
import screen.personalize.PersonalizeInformScreen
import screen.security.PinScreen
import screen.security.SetupSecurityCompletedScreen

class MockDigidWebTests : TestBase() {

    private val chosenPin = "122222"
    private val bsn = "999991771"

    @BeforeEach
    fun setUp() {
        val introductionScreen = IntroductionScreen()
        val expectationsScreen = IntroductionExpectationsScreen()
        val privacyScreen = IntroductionPrivacyScreen()
        val conditionsScreen = IntroductionConditionsScreen()
        val pinScreen = PinScreen()
        val setupSecurityCompletedScreen = SetupSecurityCompletedScreen()
        val personalizeInformScreen = PersonalizeInformScreen()

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
    }

    @Test
    fun verifyMockDigidLogin() {
        val digidLoginStartWebPage = DigidLoginStartWebPage()
        assertTrue(digidLoginStartWebPage.visible(), "digid login start web page is not visible")

        digidLoginStartWebPage.clickMockLoginButton()

        val digidLoginMockWebPage = DigidLoginMockWebPage()
        assertTrue(digidLoginMockWebPage.visible(), "digid login mock web page is not visible")

        digidLoginMockWebPage.enterBsn(bsn)
        digidLoginMockWebPage.clickLoginButton()
    }
}
