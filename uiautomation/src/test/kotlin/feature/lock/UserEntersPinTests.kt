package feature.lock

import helper.TestBase
import navigator.MenuNavigator
import navigator.OnboardingNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertAll
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Nested
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.about.AboutScreen
import screen.dashboard.DashboardScreen
import screen.error.NoInternetErrorScreen
import screen.menu.MenuScreen
import screen.security.ForgotPinScreen
import screen.security.PinScreen
import screen.security.TemporarilyBlockedScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${UserEntersPinTests.USE_CASE} User enters pin [${UserEntersPinTests.JIRA_ID}]")
class UserEntersPinTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 2.3"
        const val JIRA_ID = "PVW-869"
    }

    private lateinit var pinScreen: PinScreen
    private lateinit var temporarilyBlockedScreen: TemporarilyBlockedScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickLogoutButton()

        pinScreen = PinScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1 The User can enter a six digit PIN on an in-app keyboard. [${JIRA_ID}]")
    fun verifyPinScreenVisible(testInfo: TestInfo) {
        setUp(testInfo)
        assertTrue(pinScreen.pinScreenVisible(), "pin screen is not visible")
        assertTrue(pinScreen.pinKeyboardVisible(), "pin keyboard is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.2 The PIN is not visible at any time, only the length of the entered PIN. [${JIRA_ID}]")
    fun verifyHiddenPin(testInfo: TestInfo) {
        setUp(testInfo)
        val pin = "34567"
        pinScreen.enterPin(pin)

        assertTrue(pinScreen.enteredPinAbsent(pin), "entered pin is not absent")
    }

    /**
     * 3. The PIN does not leave the wallet app, not even in encrypted fashion.
     * >> This requirement hard, if not impossible to be tested in an e2e setup and should be validated during an audit of the app.
     */

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.5. Upon PIN entry, when the app cannot connect to the server it displays an appropriate error. [${JIRA_ID}]")
    fun verifyNotConnectedErrorMessage(testInfo: TestInfo) {
        setUp(testInfo)
        val noInternetErrorScreen = NoInternetErrorScreen()
        try {
            val pin = "122222"
            pinScreen.disableInternetConnection()
            pinScreen.enterPin(pin)
            assertAll(
                { assertTrue(noInternetErrorScreen.headlineVisible(), "Headline is not visible") },
                { assertTrue(noInternetErrorScreen.descriptionVisible(), "Description is not visible") },
                { assertTrue(noInternetErrorScreen.tryAgainButtonVisible(), "Try again button is not visible") }
            )
            noInternetErrorScreen.seeDetails()
            assertAll(
                { assertTrue(noInternetErrorScreen.appVersionLabelVisible(), "App version is not visible") },
                { assertTrue(noInternetErrorScreen.osVersionLabelVisible(), "Os version is not visible") },
                { assertTrue(noInternetErrorScreen.appConfigLabelVisible(), "appConfig is not visible") },
                { assertTrue(noInternetErrorScreen.appVersionVisible(), "App version is not visible") },
                { assertTrue(noInternetErrorScreen.osVersionVisible(), "Os version is not visible") },
                { assertTrue(noInternetErrorScreen.appConfigVisible(), "appConfig is not visible") }
            )
        } finally {
            noInternetErrorScreen.enableNetworkConnection();
        }
    }

    @Nested
    @DisplayName("$USE_CASE.6 After unsuccessful PIN entry, when the user has retries left:")
    inner class UnsuccessfulPinEntry {

        @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
        @DisplayName("$USE_CASE.6.1 (non-final round, initial attempt) if this was the initial PIN entry in the round, the app simply indicates that the PIN was wrong.  6.2/3/4. After unsuccessful PIN entry, when the user has retries left. 7. After PIN validation, when the user has a timeout, the app indicates the number of minutes the user must wait. [${JIRA_ID}]")
        fun verifyRetriesAndTimeout(testInfo: TestInfo) {
            setUp(testInfo)
            pinScreen.enterPin("123456")
            assertTrue(pinScreen.pinErrorDialogNonFinalRoundInitialAttemptVisible(), "pin error is not visible")
            pinScreen.closePinIncorrectAlertDialog()
            pinScreen.enterPin("123456")
            assertTrue(pinScreen.pinErrorDialogNonFinalRoundNonFinalAttemptVisible("2"), "pin error is not visible")
            pinScreen.closePinIncorrectAlertDialog()
            pinScreen.enterPin("123456")
            assertTrue(pinScreen.pinErrorDialogNonFinalRoundFinalAttemptVisible(), "pin error is not visible")
            pinScreen.closePinIncorrectAlertDialog()
            pinScreen.enterPin("123456")
            temporarilyBlockedScreen = TemporarilyBlockedScreen()
            assertAll(
                { assertTrue(temporarilyBlockedScreen.deleteWalletButtonVisible(), "Delete wallet button is not visible") },
                { assertTrue(temporarilyBlockedScreen.forgotPinButtonVisible(), "Forgot pin button is not visible") },
                { assertTrue(temporarilyBlockedScreen.timeoutDurationLeftVisible("57"), "Timeout duration is not visible") }
            )

        }
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.9 The app offers an entry to the ‘Forgot PIN’ flow. [${JIRA_ID}]")
    fun verifyForgotPinEntry(testInfo: TestInfo) {
        setUp(testInfo)
        pinScreen.clickForgotPinButton()

        val forgotPinScreen = ForgotPinScreen()
        assertTrue(forgotPinScreen.visible(), "forgot pin screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.10 Upon valid PIN entry, the last active screen is displayed, or the onboarding if it has not completed, or the dashboard if the app boots. [${JIRA_ID}]")
    fun verifyLastActiveScreen(testInfo: TestInfo) {
        setUp(testInfo)
        pinScreen.enterPin(OnboardingNavigator.PIN)

        val dashboardScreen = DashboardScreen()
        assertTrue(dashboardScreen.visible(), "dashboard screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.11 The PIN entry screen offers an entrance to the App Info page. [${JIRA_ID}]")
    fun verifyAppInfoButton(testInfo: TestInfo) {
        setUp(testInfo)
        pinScreen.clickAppInfoButton()

        val aboutScreen = AboutScreen()
        assertTrue(aboutScreen.visible(), "about screen is not visible")
    }
}
