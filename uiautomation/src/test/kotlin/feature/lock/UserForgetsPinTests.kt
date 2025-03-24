package feature.lock

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertAll
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.security.ForgotPinScreen
import screen.security.PinScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${UserForgetsPinTests.USE_CASE} User forgets pin [${UserForgetsPinTests.JIRA_ID}]")
class UserForgetsPinTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 2.3"
        const val JIRA_ID = "PVW-870"
    }

    private lateinit var forgotPinScreen: ForgotPinScreen

    fun setUp() {
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.PersonalizeConfirmIssuance)

        PinScreen().clickForgotPinButton()

        forgotPinScreen = ForgotPinScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1 The app explains that the user has to reset the wallet in order to regain access. 2 The app explains that upon resetting the wallet, data CANNOT be recovered. 3 The app offers an entrance to resetting the app. [${JIRA_ID}]")
    fun verifyForgotPin() {
        setUp()
        assertAll(
            { assertTrue(forgotPinScreen.visible(), "forgot pin screen is not visible") },
            { assertTrue(forgotPinScreen.dataLossTextVisible(), "data loss description text is not visible") },
            { assertTrue(forgotPinScreen.resetButtonVisible(), "reset wallet button is not visible") }
        )
    }
}
