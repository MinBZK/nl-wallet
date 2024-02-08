package feature.lock

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junitpioneer.jupiter.RetryingTest
import screen.about.AboutScreen
import screen.dashboard.DashboardScreen
import screen.security.ForgotPinScreen
import screen.security.PinScreen

@DisplayName("UC 2.3 - User enters pin [PVW-869]")
class UserEntersPinTests : TestBase() {

    private lateinit var pinScreen: PinScreen

    @BeforeEach
    fun setUp() {
        OnboardingNavigator().toScreen(OnboardingScreen.Dashboard)

        restartApp()

        pinScreen = PinScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("1. The User can enter a six digit PIN on an in-app keyboard.")
    fun verifyPinScreenVisible() {
        assertTrue(pinScreen.pinScreenVisible(), "pin screen is not visible")
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

    /**
     * 4. Upon PIN entry, when the app cannot connect to the server it displays an appropriate error.
     * >> Manual test: https://SSSS/jira/browse/PVW-1998
     */

    /**
     * 5. The app enforces the following PIN-attempt policy.
     * >> Manual test: https://SSSS/jira/browse/PVW-2018
     */

    /**
     * 6. After PIN validation, when the user has retries left, the app indicates the number of retries.
     * >> Manual test: https://SSSS/jira/browse/PVW-2019
     */

    /**
     * 7. After PIN validation, when the user has a timeout, the app indicates the number of minutes the user must wait.
     * >> Manual test: https://SSSS/jira/browse/PVW-2020
     */

    /**
     * 8. After PIN validation, when the app is blocked.
     * >> Manual test: https://SSSS/jira/browse/PVW-2021
     */

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("9. The app offers an entry to the ‘Forgot PIN’ flow.")
    fun verifyForgotPinEntry() {
        pinScreen.clickForgotPinButton()

        val forgotPinScreen = ForgotPinScreen()
        assertTrue(forgotPinScreen.visible(), "forgot pin screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("10. Upon valid PIN entry, the last active screen is displayed, or the onboarding if it has not completed, or the dashboard if the app boots.")
    fun verifyLastActiveScreen() {
        pinScreen.enterPin(OnboardingNavigator.PIN)

        val dashboardScreen = DashboardScreen();
        assertTrue(dashboardScreen.visible(), "dashboard screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("11. The PIN entry screen offers an entrance to the App Info page.")
    fun verifyAboutAppButton() {
        pinScreen.clickAboutAppButton()

        val aboutScreen = AboutScreen()
        assertTrue(aboutScreen.visible(), "about screen is not visible")
    }
}
