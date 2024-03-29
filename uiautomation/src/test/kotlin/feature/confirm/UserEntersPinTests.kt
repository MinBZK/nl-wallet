package feature.confirm

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junitpioneer.jupiter.RetryingTest
import screen.personalize.PersonalizePidPreviewScreen
import screen.personalize.PersonalizeSuccessScreen
import screen.security.ForgotPinScreen
import screen.security.PinScreen

@DisplayName("UC 2.4 - User enters pin [PVW-1119]")
class UserEntersPinTests : TestBase() {

    private lateinit var pinScreen: PinScreen

    @BeforeEach
    fun setUp() {
        OnboardingNavigator().toScreen(OnboardingScreen.PersonalizeConfirmIssuance)

        pinScreen = PinScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("1. The user can go back from the PIN screen.")
    fun verifyBackButton() {
        pinScreen.clickBackButton()

        val personalizePidPreviewScreen = PersonalizePidPreviewScreen()
        assertTrue(personalizePidPreviewScreen.visible(), "personalize pid preview screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("2. The User can enter a six digit PIN on an in-app keyboard.")
    fun verifyPinScreenVisible() {
        assertTrue(pinScreen.personalizeConfirmPinScreenVisible(), "personalize confirm pin screen is not visible")
        assertTrue(pinScreen.pinKeyboardVisible(), "pin keyboard is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("3. The PIN is not visible at any time, only the length of the entered PIN.")
    fun verifyHiddenPin() {
        val pin = "34567"
        pinScreen.enterPin(pin)
        assertTrue(pinScreen.enteredPinAbsent(pin), "entered pin is not absent")
    }

    /**
     * 4. The PIN does not leave the wallet app, not even in encrypted fashion.
     * >> This requirement hard, if not impossible to be tested in an e2e setup and should be validated during an audit of the app.
     */

    /**
     * 5. Upon PIN entry, when the app cannot connect to the server it displays an appropriate error.
     * >> Manual test: https://SSSS/jira/browse/PVW-1998
     */

    /**
     * 6. The app enforces the following PIN-attempt policy.
     * >> Manual test: https://SSSS/jira/browse/PVW-2018
     */

    /**
     * 7. After PIN validation, when the app is blocked.
     * >> Manual test: https://SSSS/jira/browse/PVW-2021
     */

    /**
     * 8. After PIN validation, when the user has retries left, the app indicates the number of retries.
     * >> Manual test: https://SSSS/jira/browse/PVW-2019
     */

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("9. The app offers an entry to the ‘Forgot PIN’ flow.")
    fun verifyForgotPinEntry() {
        pinScreen.clickForgotPinButton()

        val forgotPinScreen = ForgotPinScreen()
        assertTrue(forgotPinScreen.visible(), "forgot pin screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("10. Upon valid PIN entry, the procedure is confirmed.")
    fun verifyProcedureConfirmScreen() {
        pinScreen.enterPin(OnboardingNavigator.PIN)

        val personalizeSuccessScreen = PersonalizeSuccessScreen()
        assertTrue(personalizeSuccessScreen.visible(), "personalize success screen is not visible")
    }
}
