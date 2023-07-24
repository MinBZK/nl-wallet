package uiTests

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junitpioneer.jupiter.RetryingTest
import screens.introduction.IntroductionPrivacyPolicyScreen
import screens.introduction.IntroductionScreens

class IntroductionScreenTests : TestBase() {

    private lateinit var introductionScreens: IntroductionScreens
    private lateinit var introductionPrivacyPolicyScreen: IntroductionPrivacyPolicyScreen

    @BeforeEach
    fun setUp() {
        introductionScreens = IntroductionScreens()
        introductionPrivacyPolicyScreen = IntroductionPrivacyPolicyScreen()
    }

    @RetryingTest(value = 3, name = "{displayName} - #{index}")
    @DisplayName("UC 9.3 - Verify introduction screens and privacy policy screen")
    @Tags(Tag("smoke"), Tag("android"), Tag("iOS"))
    fun verifyIntroductionScreens() {
        introductionScreens.clickNextButton()
        introductionScreens.clickNextButton()
        introductionScreens.clickNextButton()
        introductionScreens.clickNextButton()
        introductionScreens.clickPrivacyPolicyButton()

        introductionPrivacyPolicyScreen.verifyPlaceholderScreenIsVisible()?.let {
            assertTrue(
                it,
                "placeholder screen is not visible"
            )
        }
    }

    @RetryingTest(value = 3, name = "{displayName} - #{index}")
    @DisplayName("UC 9.3 - Verify if the app start in Dutch language")
    @Tags(Tag("smoke"), Tag("android"), Tag("iOS"))
    fun verifyDutchLanguage() {
        assertEquals(introductionScreens.verifyNextButtonText(), "Volgende")
    }

    @RetryingTest(value = 3, name = "{displayName} - #{index}")
    @DisplayName("UC 9.3 - Verify if the app start in English language")
    @Tags(Tag("smoke"), Tag("android"), Tag("iOS"), Tag("english"))
    fun verifyEnglishLanguage() {
        assertEquals(introductionScreens.verifyNextButtonText(), "Next")
    }

    @RetryingTest(value = 3, name = "{displayName} - #{index}")
    @DisplayName("UC 9.3 - Verify if device language is set to france if the app start in English")
    @Tags(Tag("smoke"), Tag("android"), Tag("iOS"), Tag("france"))
    fun verifyOtherLanguage() {
        assertEquals(introductionScreens.verifyNextButtonText(), "Next")
    }
}
