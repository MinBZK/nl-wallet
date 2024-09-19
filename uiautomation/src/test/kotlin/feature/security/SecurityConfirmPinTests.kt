package feature.security

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.security.PinScreen
import screen.security.SecuritySetupCompletedScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${SecurityConfirmPinTests.USE_CASE} User confirms PIN [${SecurityConfirmPinTests.JIRA_ID}]")
class SecurityConfirmPinTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 2.1"
        const val JIRA_ID = "PVW-1216"
    }

    private val incorrectConfirmPin = "123333"

    private lateinit var pinScreen: PinScreen

    @BeforeEach
    fun setUp() {
        OnboardingNavigator().toScreen(OnboardingScreen.SecurityConfirmPin)

        pinScreen = PinScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1 The App asks the user to re-enter their PIN. [$JIRA_ID]")
    fun verifyConfirmPinScreenVisible() {
        assertTrue(pinScreen.confirmPinScreenVisible(), "confirm pin screen is not visible")
        assertTrue(pinScreen.pinKeyboardVisible(), "pin keyboard is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.2 Upon PIN entry, the App asserts that the first and second entry are equal. [$JIRA_ID]")
    fun verifyConfirmPin() {
        pinScreen.enterPin(OnboardingNavigator.PIN)

        val securitySetupCompletedScreen = SecuritySetupCompletedScreen()
        assertTrue(securitySetupCompletedScreen.visible(), "setup security completed screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.3 Upon incorrect entry, the App displays an error message and asks the user to try again. [$JIRA_ID]")
    fun verifyIncorrectConfirmPin() {
        pinScreen.enterPin(incorrectConfirmPin)
        assertTrue(pinScreen.confirmPinErrorMismatchVisible(), " confirm pin error mismatch is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.4 The user may attempt entering their PIN in 2 attempts. [$JIRA_ID]")
    fun verifyIncorrectConfirmPinTwice() {
        pinScreen.enterPin(incorrectConfirmPin)
        pinScreen.closeAlertDialog()
        pinScreen.enterPin(incorrectConfirmPin)
        assertTrue(pinScreen.confirmPinErrorMismatchFatalVisible(), "confirm pin error fatal mismatch is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.5 After 2 attempts, the App offers the user to pick a new PIN. [$JIRA_ID]")
    fun verifyRestartChoosePin() {
        pinScreen.enterPin(incorrectConfirmPin)
        pinScreen.closeAlertDialog()
        pinScreen.enterPin(incorrectConfirmPin)
        pinScreen.clickConfirmPinErrorFatalCta()
        assertTrue(pinScreen.choosePinScreenVisible(), "choose pin screen is not visible")
    }
}
