package feature.confirm

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertAll
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.error.NoInternetErrorScreen
import screen.personalize.PersonalizePidPreviewScreen
import screen.personalize.PersonalizeSuccessScreen
import screen.security.ForgotPinScreen
import screen.security.PinScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${UserEntersPinTests.USE_CASE} User enters pin [${UserEntersPinTests.JIRA_ID}]")
class UserEntersPinTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 2.4"
        const val JIRA_ID = "PVW-1119"
    }

    private lateinit var pinScreen: PinScreen
    private lateinit var noInternetErrorScreen: NoInternetErrorScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.PersonalizeConfirmIssuance)
        pinScreen = PinScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1 The user can go back from the PIN screen. [${JIRA_ID}]")
    fun verifyBackButton(testInfo: TestInfo) {
        setUp(testInfo)
        pinScreen.clickBackButton()

        val personalizePidPreviewScreen = PersonalizePidPreviewScreen()
        assertTrue(personalizePidPreviewScreen.visible(), "personalize pid preview screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.2 The User can enter a six digit PIN on an in-app keyboard. [${JIRA_ID}]")
    fun verifyPinScreenVisible(testInfo: TestInfo) {
        setUp(testInfo)
        assertTrue(pinScreen.personalizeConfirmPinScreenVisible(), "personalize confirm pin screen is not visible")
        assertTrue(pinScreen.pinKeyboardVisible(), "pin keyboard is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.3 The PIN is not visible at any time, only the length of the entered PIN. [${JIRA_ID}]")
    fun verifyHiddenPin(testInfo: TestInfo) {
        setUp(testInfo)
        val pin = "34567"
        pinScreen.enterPin(pin)
        assertTrue(pinScreen.enteredPinAbsent(pin), "entered pin is not absent")
    }

    /**
     * 4. The PIN does not leave the wallet app, not even in encrypted fashion.
     * >> This requirement hard, if not impossible to be tested in an e2e setup and should be validated during an audit of the app.
     */

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.5. Upon PIN entry, when the app cannot connect to the server it displays an appropriate error. [${JIRA_ID}]")
    fun verifyNotConnectedErrorMessage(testInfo: TestInfo) {
        try {
            setUp(testInfo)
            val pin = "122222"
            noInternetErrorScreen = NoInternetErrorScreen()
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
    @DisplayName("$USE_CASE.9 The app offers an entry to the ‘Forgot PIN’ flow. [${JIRA_ID}]")
    fun verifyForgotPinEntry(testInfo: TestInfo) {
        setUp(testInfo)
        pinScreen.clickForgotPinButton()

        val forgotPinScreen = ForgotPinScreen()
        assertTrue(forgotPinScreen.visible(), "forgot pin screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.10 Upon valid PIN entry, the procedure is confirmed. [${JIRA_ID}]")
    fun verifyProcedureConfirmScreen(testInfo: TestInfo) {
        setUp(testInfo)
        pinScreen.enterPin(OnboardingNavigator.PIN)

        val personalizeSuccessScreen = PersonalizeSuccessScreen()
        assertTrue(personalizeSuccessScreen.visible(), "personalize success screen is not visible")
    }
}
