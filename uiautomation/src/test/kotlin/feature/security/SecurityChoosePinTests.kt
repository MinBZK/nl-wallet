package feature.security

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junitpioneer.jupiter.RetryingTest
import screen.about.AboutScreen
import screen.security.PinScreen

@DisplayName("UC 2.1 - User chooses PIN [PVW-1215]")
class SecurityChoosePinTests : TestBase() {

    private lateinit var pinScreen: PinScreen

    @BeforeEach
    fun setUp() {
        OnboardingNavigator().toScreen(OnboardingScreen.SecurityChoosePin)

        pinScreen = PinScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("1. The User can enter a six digit PIN on an in-app keyboard.")
    fun verifyChoosePinScreenVisible() {
        assertTrue(pinScreen.choosePinScreenVisible(), "choose pin screen is not visible")
        assertTrue(pinScreen.pinKeyboardVisible(), "pin keyboard is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("2. The PIN is not visible at any time, only the length of the entered PIN.")
    fun verifyHiddenPin() {
        val pin = "34567"
        pinScreen.enterPin(pin)
        assertTrue(pinScreen.enteredPinAbsent(pin), "entered pin is not absent")
    }

    /**
     * 3. The PIN does not leave the wallet app, not even in encrypted fashion.
     * >> This requirement hard, if not impossible to be tested in an e2e setup and should be validated during an audit of the app.
     */

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("4. Upon PIN entry, the app checks that the PIN matches the security requirements.")
    fun verifyPinTwoUniqueDigitsError() {
        pinScreen.enterPin("111111")
        assertTrue(
            pinScreen.choosePinErrorTooFewUniqueDigitsVisible(),
            "choose pin error too few unique digits is not visible"
        )
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
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

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("6. Upon successful PIN entry, go to Feature 'User confirms PIN'.")
    fun verifySuccessfulPinEntry() {
        pinScreen.enterPin("122222")
        assertTrue(pinScreen.confirmPinScreenVisible(), "confirm pin screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("7. The screen offers an entrance to the App Info screen.")
    fun verifyAboutAppButton() {
        pinScreen.clickAboutAppButton()

        val aboutScreen = AboutScreen()
        assertTrue(aboutScreen.visible(), "about screen is not visible")
    }
}
