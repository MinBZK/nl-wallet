package uiTests

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junitpioneer.jupiter.RetryingTest
import screen.introduction.IntroductionConditionsScreen
import screen.introduction.IntroductionExpectationsScreen
import screen.introduction.IntroductionPrivacyScreen
import screen.introduction.IntroductionScreen

class IntroductionScreenTests : TestBase() {

    private lateinit var introductionScreen: IntroductionScreen
    private lateinit var expectationsScreen: IntroductionExpectationsScreen
    private lateinit var privacyScreen: IntroductionPrivacyScreen
    private lateinit var conditionsScreen: IntroductionConditionsScreen

    @BeforeEach
    fun setUp() {
        introductionScreen = IntroductionScreen()
        expectationsScreen = IntroductionExpectationsScreen()
        privacyScreen = IntroductionPrivacyScreen()
        conditionsScreen = IntroductionConditionsScreen()
    }

    @RetryingTest(value = 3, name = "{displayName} - #{index}")
    @DisplayName("PVW-1218 - App displays introductory information [1.1]")
    @Tags(Tag("smoke"), Tag("android"), Tag("ios"))
    fun verifyIntroductionScreens() {
        introductionScreen.clickNextButton()
        introductionScreen.clickNextButton()
        introductionScreen.clickNextButton()
        introductionScreen.clickNextButton()

        assertTrue(
            expectationsScreen.waitForButtonVisibility(),
            "expectations screen not visible"
        )
    }

    @RetryingTest(value = 3, name = "{displayName} - #{index}")
    @DisplayName("PVW-1220 - App displays privacy statement [1.1]")
    @Tags(Tag("smoke"), Tag("android"), Tag("ios"))
    fun verifyPrivacyConditionsScreens() {
        introductionScreen.clickSkipButton()
        expectationsScreen.clickNextButton()
        privacyScreen.clickNextButton()

        assertTrue(
            conditionsScreen.waitForButtonVisibility(),
            "conditions screen not visible"
        )
    }

    @RetryingTest(value = 3, name = "{displayName} - #{index}")
    @DisplayName("PVW-1223 - Open the App [1.2] - Verify if device language is set to Dutch, then the app starts in Dutch")
    @Tags(Tag("smoke"), Tag("android"), Tag("ios"))
    fun verifyDutchLanguage() {
        assertEquals(introductionScreen.readNextButtonText(), "Volgende")
    }

    @RetryingTest(value = 3, name = "{displayName} - #{index}")
    @DisplayName("PVW-1223 - Open the App [1.2] - Verify if device language is set to English, then the app starts in English")
    @Tags(Tag("smoke"), Tag("android"), Tag("ios"), Tag("english"))
    fun verifyEnglishLanguage() {
        assertEquals(introductionScreen.readNextButtonText(), "Next")
    }

    @RetryingTest(value = 3, name = "{displayName} - #{index}")
    @DisplayName("PVW-1223 - Open the App [1.2] - Verify if device language is set to France, then the app starts in English")
    @Tags(Tag("smoke"), Tag("android"), Tag("ios"), Tag("france"))
    fun verifyOtherLanguage() {
        assertEquals(introductionScreen.readNextButtonText(), "Next")
    }
}
