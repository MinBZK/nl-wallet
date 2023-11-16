package feature.introduction

import helper.TestBase
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Test
import screen.common.PlaceholderScreen
import screen.introduction.IntroductionConditionsScreen
import screen.introduction.IntroductionExpectationsScreen
import screen.introduction.IntroductionPrivacyScreen
import screen.introduction.IntroductionScreen
import screen.security.PinScreen

@DisplayName("UC 1.1 - User accepts terms & conditions [PVW-1221]")
class IntroductionConditionsScreenTests : TestBase() {

    private lateinit var conditionsScreen: IntroductionConditionsScreen

    @BeforeEach
    fun setUp() {
        val introductionScreen = IntroductionScreen()
        val expectationsScreen = IntroductionExpectationsScreen()
        val privacyScreen = IntroductionPrivacyScreen()

        // Start all tests on conditions screen
        introductionScreen.clickSkipButton()
        expectationsScreen.clickNextButton()
        privacyScreen.clickNextButton()

        conditionsScreen = IntroductionConditionsScreen()
    }

    @Test
    @DisplayName("1. The App displays the summary of the terms & conditions.")
    fun verifyConditionsScreen() {
        assertTrue(conditionsScreen.visible(), "expectations screen is not visible")
    }

    @Test
    @DisplayName("2. The App offers an entrance to the full terms & conditions, which is embedded in the app.")
    fun verifyConditionsButton() {
        conditionsScreen.clickConditionsButton()

        val placeholderScreen = PlaceholderScreen()
        assertTrue(placeholderScreen.visible(), "placeholder screen is not visible")
    }

    @Test
    @DisplayName("3. The App offers an option to accept the terms and conditions, leading to setup pin")
    fun verifyNextButton() {
        conditionsScreen.clickNextButton()

        val pinScreen = PinScreen()
        assertTrue(pinScreen.choosePinScreenVisible(), "choose pin screen is not visible")
    }

    @Test
    @DisplayName("4. The App offers a return to the previous screen.")
    fun verifyBackButton() {
        conditionsScreen.clickBackButton()
        assertTrue(conditionsScreen.absent(), "conditions screen is visible")
    }
}
