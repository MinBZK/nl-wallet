package feature.security

import helper.TestBase
import navigator.MenuNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junit.jupiter.api.assertAll
import org.junitpioneer.jupiter.RetryingTest
import screen.about.AboutScreen
import screen.dashboard.DashboardScreen
import screen.error.NoInternetErrorScreen
import screen.menu.MenuScreen
import screen.security.ForgotPinScreen
import screen.security.PinScreen
import screen.security.TemporarilyBlockedScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("UC2.3 Unlock the app")
class UserEntersPinTests : TestBase() {

    private lateinit var pinScreen: PinScreen
    private lateinit var temporarilyBlockedScreen: TemporarilyBlockedScreen
    private lateinit var aboutScreen: AboutScreen
    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var forgotPinScreen: ForgotPinScreen
    private lateinit var noInternetErrorScreen: NoInternetErrorScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickLogoutButton()

        pinScreen = PinScreen()
        aboutScreen = AboutScreen()
        dashboardScreen = DashboardScreen()
        forgotPinScreen = ForgotPinScreen()
        temporarilyBlockedScreen= TemporarilyBlockedScreen()
        noInternetErrorScreen = NoInternetErrorScreen()

    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC47 Unlock app with correct PIN")
    fun verifyPinScreenVisible(testInfo: TestInfo) {
        setUp(testInfo)
        assertTrue(pinScreen.pinScreenVisible(), "pin screen is not visible")
        assertTrue(pinScreen.pinKeyboardVisible(), "pin keyboard is not visible")
        val pin = "12222"
        pinScreen.enterPin(pin)
        assertTrue(pinScreen.enteredPinAbsent(pin), "entered pin is not absent")
        pinScreen.enterPin("2")
        assertTrue(dashboardScreen.visible(), "dashboard screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("Upon PIN entry, when the app cannot connect to the server it displays an appropriate error.")
    @Tags(Tag("a11yBatch2"))
    fun verifyNotConnectedErrorMessage(testInfo: TestInfo) {
        setUp(testInfo)
        try {
            pinScreen.disableInternetConnection()
            pinScreen.enterPin(DEFAULT_PIN)
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
            noInternetErrorScreen.enableInternetConnection();
        }
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC50 Unlock app with invalid PIN")
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

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC51 User selects forgot PIN")
    fun verifyForgotPinEntry(testInfo: TestInfo) {
        setUp(testInfo)
        pinScreen.clickForgotPinButton()
        assertAll(
            { assertTrue(forgotPinScreen.visible(), "forgot pin screen is not visible") },
            { assertTrue(forgotPinScreen.descriptionTextVisible(), "description text is not visible") },
            { assertTrue(forgotPinScreen.recoverButtonVisible(), "reset wallet button is not visible") }
        )
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC47 The PIN entry screen offers an entrance to the App Info page.")
    fun verifyAppInfoButton(testInfo: TestInfo) {
        setUp(testInfo)
        pinScreen.clickAppInfoButton()
        assertTrue(aboutScreen.visible(), "about screen is not visible")
    }
}
