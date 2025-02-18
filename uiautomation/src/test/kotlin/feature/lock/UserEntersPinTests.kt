package feature.lock

import helper.TestBase
import navigator.MenuNavigator
import navigator.OnboardingNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Nested
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.about.AboutScreen
import screen.dashboard.DashboardScreen
import screen.menu.MenuScreen
import screen.security.ForgotPinScreen
import screen.security.PinScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${UserEntersPinTests.USE_CASE} User enters pin [${UserEntersPinTests.JIRA_ID}]")
class UserEntersPinTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 2.3"
        const val JIRA_ID = "PVW-869"
    }

    private lateinit var pinScreen: PinScreen

    fun setUp() {
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickLogoutButton()

        pinScreen = PinScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1 The User can enter a six digit PIN on an in-app keyboard. [${JIRA_ID}]")
    fun verifyPinScreenVisible() {
        setUp()
        assertTrue(pinScreen.pinScreenVisible(), "pin screen is not visible")
        assertTrue(pinScreen.pinKeyboardVisible(), "pin keyboard is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.2 The PIN is not visible at any time, only the length of the entered PIN. [${JIRA_ID}]")
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

    /**
     * 4. Upon PIN entry, when the app cannot connect to the server it displays an appropriate error.
     * >> Manual test: https://SSSS/jira/browse/PVW-1998
     */

    /**
     * 5. The app enforces the following PIN-attempt policy.
     * >> Manual test: https://SSSS/jira/browse/PVW-2018
     */

    @Nested
    @DisplayName("$USE_CASE.6 After unsuccessful PIN entry, when the user has retries left:")
    inner class UnsuccessfulPinEntry {

        @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
        @DisplayName("$USE_CASE.6.1 (non-final round, initial attempt) if this was the initial PIN entry in the round, the app simply indicates that the PIN was wrong. [${JIRA_ID}]")
        fun verifyNonFinalRoundInitialAttempt() {
            setUp()
            pinScreen.enterPin("123456")

            assertTrue(pinScreen.pinErrorDialogNonFinalRoundInitialAttemptVisible(), "pin error is not visible")
        }

        /**
         * 6.2/3/4. After unsuccessful PIN entry, when the user has retries left.
         * >> Manual test: https://SSSS/jira/browse/PVW-2019
         */
    }

    /**
     * 7. After PIN validation, when the user has a timeout, the app indicates the number of minutes the user must wait.
     * >> Manual test: https://SSSS/jira/browse/PVW-2020
     */

    /**
     * 8. After PIN validation, when the app is blocked.
     * >> Manual test: https://SSSS/jira/browse/PVW-2021
     */

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.9 The app offers an entry to the ‘Forgot PIN’ flow. [${JIRA_ID}]")
    fun verifyForgotPinEntry() {
        setUp()
        pinScreen.clickForgotPinButton()

        val forgotPinScreen = ForgotPinScreen()
        assertTrue(forgotPinScreen.visible(), "forgot pin screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.10 Upon valid PIN entry, the last active screen is displayed, or the onboarding if it has not completed, or the dashboard if the app boots. [${JIRA_ID}]")
    fun verifyLastActiveScreen() {
        setUp()
        pinScreen.enterPin(OnboardingNavigator.PIN)

        val dashboardScreen = DashboardScreen()
        assertTrue(dashboardScreen.visible(), "dashboard screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.11 The PIN entry screen offers an entrance to the App Info page. [${JIRA_ID}]")
    fun verifyAppInfoButton() {
        setUp()
        pinScreen.clickAppInfoButton()

        val aboutScreen = AboutScreen()
        assertTrue(aboutScreen.visible(), "about screen is not visible")
    }
}
