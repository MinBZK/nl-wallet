package feature.security

import helper.TestBase
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Test
import screen.introduction.IntroductionConditionsScreen
import screen.introduction.IntroductionExpectationsScreen
import screen.introduction.IntroductionPrivacyScreen
import screen.introduction.IntroductionScreen
import screen.security.PinScreen
import screen.security.SetupSecurityCompletedScreen

@DisplayName("UC 2.1 - User confirms PIN [PVW-1216]")
class ConfirmPinTests : TestBase() {

    private val chosenPin = "122222"
    private val incorrectConfirmPin = "123333"

    private lateinit var pinScreen: PinScreen

    @BeforeEach
    fun setUp() {
        val introductionScreen = IntroductionScreen()
        val expectationsScreen = IntroductionExpectationsScreen()
        val privacyScreen = IntroductionPrivacyScreen()
        val conditionsScreen = IntroductionConditionsScreen()

        // Start all tests on confirm pin screen
        introductionScreen.clickSkipButton()
        expectationsScreen.clickNextButton()
        privacyScreen.clickNextButton()
        conditionsScreen.clickNextButton()

        pinScreen = PinScreen()
        pinScreen.enterPin(chosenPin)
    }

    @Test
    @DisplayName("1. The App asks the user to re-enter their PIN.")
    fun verifyConfirmPinScreenVisible() {
        assertTrue(pinScreen.confirmPinScreenVisible(), "confirm pin screen is not visible")
        assertTrue(pinScreen.pinKeyboardVisible(), "pin keyboard is not visible")
    }

    @Test
    @DisplayName("2. Upon PIN entry, the App asserts that the first and second entry are equal.")
    fun verifyConfirmPin() {
        pinScreen.enterPin(chosenPin)

        val setupSecurityCompletedScreen = SetupSecurityCompletedScreen()
        assertTrue(setupSecurityCompletedScreen.visible(), "setup security completed screen is not visible")
    }

    @Test
    @DisplayName("3. Upon incorrect entry, the App displays an error message and asks the user to try again.")
    fun verifyIncorrectConfirmPin() {
        pinScreen.enterPin(incorrectConfirmPin)
        assertTrue(pinScreen.confirmPinErrorMismatchVisible(), " confirm pin error mismatch is not visible")
    }

    @Test
    @DisplayName("4. The user may attempt entering their PIN in 2 attempts.")
    fun verifyIncorrectConfirmPinTwice() {
        pinScreen.enterPin(incorrectConfirmPin)
        pinScreen.enterPin(incorrectConfirmPin)
        assertTrue(pinScreen.confirmPinErrorMismatchFatalVisible(), "confirm pin error fatal mismatch is not visible")
    }

    @Test
    @DisplayName("5. After 2 attempts, the App offers the user to pick a new PIN.")
    fun verifyRestartChoosePin() {
        pinScreen.enterPin(incorrectConfirmPin)
        pinScreen.enterPin(incorrectConfirmPin)
        pinScreen.clickConfirmPinErrorFatalCta()
        assertTrue(pinScreen.choosePinScreenVisible(), "choose pin screen is not visible")
    }
}
