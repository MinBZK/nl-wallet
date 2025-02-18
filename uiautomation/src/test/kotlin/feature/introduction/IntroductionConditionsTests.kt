package feature.introduction

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.introduction.IntroductionConditionsScreen
import screen.security.PinScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${IntroductionConditionsTests.USE_CASE} User accepts terms & conditions [${IntroductionConditionsTests.JIRA_ID}]")
class IntroductionConditionsTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 1.1"
        const val JIRA_ID = "PVW-1221"
    }

    private lateinit var conditionsScreen: IntroductionConditionsScreen

    fun setUp() {
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.IntroductionConditions)

        conditionsScreen = IntroductionConditionsScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1 The App displays the summary of the terms & conditions. [${JIRA_ID}]")
    fun verifyConditionsScreen() {
        setUp()
        assertTrue(conditionsScreen.visible(), "expectations screen is not visible")
    }

    /**
     * 2. The App offers an entrance to the full terms & conditions, which is embedded in the app.
     * >> This AC is under review by legal experts; meanwhile the implementation of the app has changed (removed the conditions button) until further notice.
     */

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.3 The App offers an option to accept the terms and conditions, leading to setup pin. [${JIRA_ID}]")
    fun verifyNextButton() {
        setUp()
        conditionsScreen.clickNextButton()

        val pinScreen = PinScreen()
        assertTrue(pinScreen.choosePinScreenVisible(), "choose pin screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.4 The App offers a return to the previous screen. [${JIRA_ID}]")
    fun verifyBackButton() {
        setUp()
        conditionsScreen.clickBackButton()
        assertTrue(conditionsScreen.absent(), "conditions screen is visible")
    }
}
