package feature.security

import helper.TestBase
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Test
import screen.about.AboutScreen
import screen.introduction.IntroductionConditionsScreen
import screen.introduction.IntroductionExpectationsScreen
import screen.introduction.IntroductionPrivacyScreen
import screen.introduction.IntroductionScreen
import screen.security.PinScreen

@DisplayName("UC 2.1 - User chooses PIN [PVW-1215]")
class ChoosePinTests : TestBase() {

    private lateinit var pinScreen: PinScreen

    @BeforeEach
    fun setUp() {
        val introductionScreen = IntroductionScreen()
        val expectationsScreen = IntroductionExpectationsScreen()
        val privacyScreen = IntroductionPrivacyScreen()
        val conditionsScreen = IntroductionConditionsScreen()

        // Start all tests on select pin screen
        introductionScreen.clickSkipButton()
        expectationsScreen.clickNextButton()
        privacyScreen.clickNextButton()
        conditionsScreen.clickNextButton()

        pinScreen = PinScreen()
    }

    @Test
    @DisplayName("1. The User can enter a six digit PIN on an in-app keyboard.")
    fun verifySelectPinScreenVisible() {
        assertTrue(pinScreen.choosePinScreenVisible(), "choose pin screen is not visible")
        assertTrue(pinScreen.pinKeyboardVisible(), "pin keyboard is not visible")
    }

    @Test
    @DisplayName("2. The PIN is not visible at any time, only the length of the entered PIN.")
    fun verifyHiddenPin() {
        val pin = "34567"
        pinScreen.enterPin(pin)
        assertTrue(pinScreen.enteredPinAbsent(pin), "entered pin is not absent")
    }

    @Test
    @DisplayName("3. The PIN does not leave the wallet app, not even in encrypted fashion.")
    fun verifyPinNotLeavingApp() {
        // This requirement hard, if not impossible to be tested in an e2e setup and should be validated during an audit of the app.
    }

    @Test
    @DisplayName("4. Upon PIN entry, the app checks that the PIN matches the security requirements.")
    fun verifyPinTwoUniqueDigitsError() {
        pinScreen.enterPin("111111")
        assertTrue(
            pinScreen.choosePinErrorTooFewUniqueDigitsVisible(),
            "choose pin error too few unique digits is not visible"
        )
    }

    @Test
    @DisplayName("5. Upon failing the security requirements, the App rejects the PIN and explains why.")
    fun verifyPinAscendingDescendingError() {
        pinScreen.enterPin("123456")
        assertTrue(
            pinScreen.choosePinErrorSequentialDigitsVisible(),
            "choose pin error sequential digits is not visible"
        )

        pinScreen.enterPin("987654")
        assertTrue(
            pinScreen.choosePinErrorSequentialDigitsVisible(),
            "choose pin error sequential digits is not visible"
        )
    }

    @Test
    @DisplayName("6. Upon successful PIN entry, go to Feature 'User confirms PIN'.")
    fun verifySuccessfulPinEntry() {
        pinScreen.enterPin("122222")
        assertTrue(pinScreen.confirmPinScreenVisible(), "confirm pin screen is not visible")
    }

    @Test
    @DisplayName("7. The screen offers an entrance to the App Info screen.")
    fun verifyAboutAppButton() {
        pinScreen.clickAboutAppButton()

        val aboutScreen = AboutScreen()
        assertTrue(aboutScreen.visible(), "about screen is not visible")
    }
}
