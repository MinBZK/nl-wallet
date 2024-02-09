package feature.introduction

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junitpioneer.jupiter.RetryingTest
import screen.common.PlaceholderScreen
import screen.introduction.IntroductionConditionsScreen
import screen.security.PinScreen

@DisplayName("UC 1.1 - User accepts terms & conditions [PVW-1221]")
class IntroductionConditionsScreenTests : TestBase() {

    private lateinit var conditionsScreen: IntroductionConditionsScreen

    @BeforeEach
    fun setUp() {
        OnboardingNavigator().toScreen(OnboardingScreen.IntroductionConditions)

        conditionsScreen = IntroductionConditionsScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("1. The App displays the summary of the terms & conditions.")
    fun verifyConditionsScreen() {
        assertTrue(conditionsScreen.visible(), "expectations screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("2. The App offers an entrance to the full terms & conditions, which is embedded in the app.")
    fun verifyConditionsButton() {
        conditionsScreen.clickConditionsButton()

        val placeholderScreen = PlaceholderScreen()
        assertTrue(placeholderScreen.visible(), "placeholder screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("3. The App offers an option to accept the terms and conditions, leading to setup pin")
    fun verifyNextButton() {
        conditionsScreen.clickNextButton()

        val pinScreen = PinScreen()
        assertTrue(pinScreen.choosePinScreenVisible(), "choose pin screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("4. The App offers a return to the previous screen.")
    fun verifyBackButton() {
        conditionsScreen.clickBackButton()
        assertTrue(conditionsScreen.absent(), "conditions screen is visible")
    }
}
