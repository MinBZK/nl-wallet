package feature.introduction

import helper.TestBase
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Nested
import org.junitpioneer.jupiter.RetryingTest
import screen.introduction.IntroductionExpectationsScreen
import screen.introduction.IntroductionScreen

@DisplayName("UC 1.1 - App displays introductory information [PVW-1218]")
class IntroductionScreenTests : TestBase() {

    private lateinit var introductionScreen: IntroductionScreen

    @BeforeEach
    fun setUp() {
        introductionScreen = IntroductionScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("1. The App shows a welcome screen so the user knows they are using the NL wallet.")
    fun verifyWelcomeScreen() {
        assertTrue(introductionScreen.page1Visible(), "page 1 is not visible")
    }

    @Nested
    @DisplayName("2. The App shows a series of introductory screens explaining the following:")
    inner class IntroductoryScreens {

        @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
        @DisplayName("2.1. The purpose of the app (online identification and data sharing)")
        fun verifyPurposeScreen() {
            introductionScreen.clickNextButton() // page 1 -> 2
            assertTrue(introductionScreen.page2Visible(), "page 2 is not visible")
        }

        @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
        @DisplayName("2.2. The privacy benefits of the app (selective disclosure)")
        fun verifyPrivacyScreen() {
            introductionScreen.clickNextButton() // page 1 -> 2
            introductionScreen.clickNextButton() // page 2 -> 3
            assertTrue(introductionScreen.page3Visible(), "page 3 is not visible")
        }

        @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
        @DisplayName("2.3. The autonomy benefits of the app (user in control, transparency, history)")
        fun verifyAutonomyScreen() {
            introductionScreen.clickNextButton() // page 1 -> 2
            introductionScreen.clickNextButton() // page 2 -> 3
            introductionScreen.clickNextButton() // page 3 -> 4
            assertTrue(introductionScreen.page4Visible(), "page 4 is not visible")

            introductionScreen.clickNextButton() // page 4 -> next screen
            assertTrue(introductionScreen.page4Absent(), "page 4 is visible")
        }
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("3. The App offers a button to skip the intro, leading to the onboarding process summary.")
    fun verifySkipIntroButtons() {
        val expectationsScreen = IntroductionExpectationsScreen()

        assertTrue(introductionScreen.page1Visible(), "page 1 is not visible")

        // Skip from page 1
        introductionScreen.clickSkipButton() // page 1 -> expectations
        assertTrue(expectationsScreen.visible(), "expectations screen is not visible")

        // Back to page 1, next, skip from page 2
        expectationsScreen.clickBackButton() // expectations -> page 1
        introductionScreen.clickNextButton() // page 1 -> page 2
        assertTrue(introductionScreen.page2Visible(), "page 2 is not visible")
        introductionScreen.clickSkipButton() // page 2 -> expectations
        assertTrue(expectationsScreen.visible(), "expectations screen is not visible")

        // Back to page 2, next, skip from page 3
        expectationsScreen.clickBackButton() // expectations -> page 2
        introductionScreen.clickNextButton() // page 2 -> page 3
        assertTrue(introductionScreen.page3Visible(), "page 3 is not visible")
        introductionScreen.clickSkipButton() // page 3 -> expectations
        assertTrue(expectationsScreen.visible(), "expectations screen is not visible")

        // Back to page 3, next, skip from page 4
        expectationsScreen.clickBackButton() // expectations -> page 3
        introductionScreen.clickNextButton() // page 3 -> page 4
        assertTrue(introductionScreen.page4Visible(), "page 4 is not visible")
        introductionScreen.clickSkipButton() // page 4 -> expectations
        assertTrue(expectationsScreen.visible(), "expectations screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("4. These explanation screens all display a back-button.")
    fun verifyPageBackButtons() {
        introductionScreen.clickNextButton() // page 1 -> 2
        introductionScreen.clickNextButton() // page 2 -> 3
        introductionScreen.clickNextButton() // page 3 -> 4
        assertTrue(introductionScreen.page4Visible(), "page 4 is not visible")

        introductionScreen.clickBackButton() // page 4 -> 3
        assertTrue(introductionScreen.page3Visible(), "page 3 is not visible")

        introductionScreen.clickBackButton() // page 3 -> 2
        assertTrue(introductionScreen.page2Visible(), "page 2 is not visible")

        introductionScreen.clickBackButton() // page 2 -> 1
        assertTrue(introductionScreen.page1Visible(), "page 1 is not visible")

        introductionScreen.clickBackButton() // nothing happens
        assertTrue(introductionScreen.page1Visible(), "page 1 is not visible")
    }
}
