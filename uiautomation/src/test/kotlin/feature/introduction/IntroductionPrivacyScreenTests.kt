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

@DisplayName("UC 1.1 - App displays privacy statement [PVW-1220]")
class IntroductionPrivacyScreenTests : TestBase() {

    private lateinit var privacyScreen: IntroductionPrivacyScreen

    @BeforeEach
    fun setUp() {
        val introductionScreen = IntroductionScreen()
        val expectationsScreen = IntroductionExpectationsScreen()

        // Start all tests on privacy screen
        introductionScreen.clickSkipButton()
        expectationsScreen.clickNextButton()

        privacyScreen = IntroductionPrivacyScreen()
    }

    @Test
    @DisplayName("1. The App displays the summary of the privacy statement.")
    fun verifyPrivacyScreen() {
        assertTrue(privacyScreen.visible(), "privacy screen is not visible")
    }

    @Test
    @DisplayName("2. The App offers an entrance to the full privacy statement, which is embedded in the app.")
    fun verifyPrivacyPolicyButton() {
        privacyScreen.clickPrivacyButton()

        val placeholderScreen = PlaceholderScreen()
        assertTrue(placeholderScreen.visible(), "placeholder screen is not visible")
    }

    @Test
    @DisplayName("3. The User can proceed to terms & conditions.")
    fun verifyNextButton() {
        privacyScreen.clickNextButton()

        val conditionsScreen = IntroductionConditionsScreen()
        assertTrue(conditionsScreen.visible(), "conditions screen is not visible")
    }

    @Test
    @DisplayName("4. The App offers a return to the previous screen.")
    fun verifyBackButton() {
        privacyScreen.clickBackButton()
        assertTrue(privacyScreen.absent(), "privacy screen is visible")
    }
}
