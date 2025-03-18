package feature.security

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.about.AboutScreen
import screen.security.PinScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${SecurityChoosePinTests.USE_CASE} User chooses PIN [${SecurityChoosePinTests.JIRA_ID}]")
class SecurityChoosePinTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 2.1"
        const val JIRA_ID = "PVW-1215"
    }

    private lateinit var pinScreen: PinScreen

    fun setUp() {
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.SecurityChoosePin)

        pinScreen = PinScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1 The User can enter a six digit PIN on an in-app keyboard. [$JIRA_ID]")
    fun verifyChoosePinScreenVisible() {
        setUp()
        assertTrue(pinScreen.choosePinScreenVisible(), "choose pin screen is not visible")
        assertTrue(pinScreen.pinKeyboardVisible(), "pin keyboard is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.2 The PIN is not visible at any time, only the length of the entered PIN. [$JIRA_ID]")
    fun verifyHiddenPin() {
        setUp()
        val pin = "34567"
        pinScreen.enterPin(pin)
        assertTrue(pinScreen.enteredPinAbsent(pin), "entered pin is not absent")
    }

    /**
     * 3. The PIN does not leave the wallet app, not even in encrypted fashion.
     * >> This requirement hard, if not impossible to be tested in an e2e setup and should be validated during an audit of the app.
     */

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.4 Upon PIN entry, the app checks that the PIN matches the security requirements. [$JIRA_ID]")
    fun verifyPinTwoUniqueDigitsError() {
        setUp()
        pinScreen.enterPin("111111")
        assertTrue(
            pinScreen.choosePinErrorTooFewUniqueDigitsVisible(),
            "choose pin error too few unique digits is not visible"
        )
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.5 Upon failing the security requirements, the App rejects the PIN and explains why. [$JIRA_ID]")
    fun verifyPinAscendingDescendingError() {
        setUp()
        pinScreen.enterPin("123456")
        assertTrue(
            pinScreen.choosePinErrorSequentialDigitsVisible(),
            "choose pin error sequential digits is not visible"
        )

        pinScreen.closeAlertDialog()
        pinScreen.enterPin("987654")
        assertTrue(
            pinScreen.choosePinErrorSequentialDigitsVisible(),
            "choose pin error sequential digits is not visible"
        )
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.6 Upon successful PIN entry, go to Feature 'User confirms PIN'. [$JIRA_ID]")
    fun verifySuccessfulPinEntry() {
        setUp()
        pinScreen.enterPin("122222")
        assertTrue(pinScreen.confirmPinScreenVisible(), "confirm pin screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.7 The screen offers an entrance to the App Info screen. [$JIRA_ID]")
    fun verifyAppInfoButton() {
        setUp()
        pinScreen.clickAppInfoButton()

        val aboutScreen = AboutScreen()
        assertTrue(aboutScreen.visible(), "about screen is not visible")
    }
}
