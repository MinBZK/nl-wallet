package feature.introduction

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.OnboardingScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junitpioneer.jupiter.RetryingTest
import screen.introduction.IntroductionExpectationsScreen
import screen.introduction.IntroductionPrivacyScreen

@DisplayName("UC 1.1 - App displays onboarding process summary [PVW-1219]")
class IntroductionExpectationsScreenTests : TestBase() {

    private lateinit var expectationsScreen: IntroductionExpectationsScreen

    @BeforeEach
    fun setUp() {
        OnboardingNavigator().toScreen(OnboardingScreen.IntroductionExpectations)

        expectationsScreen = IntroductionExpectationsScreen()
    }

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("1. The App displays the steps of the onboarding process.")
    fun verifyExpectationsScreen() {
        assertTrue(expectationsScreen.visible(), "expectations screen is not visible")
    }

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("2. The screen has a button to start the onboarding process, leading to privacy statement.")
    fun verifyNextButton() {
        expectationsScreen.clickNextButton()

        val privacyScreen = IntroductionPrivacyScreen()
        assertTrue(privacyScreen.visible(), "privacy screen is not visible")
    }
}
