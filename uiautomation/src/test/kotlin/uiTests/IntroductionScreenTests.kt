package uiTests

import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junitpioneer.jupiter.RetryingTest
import screens.introduction.IntroductionChangeLanguageScreen
import screens.introduction.IntroductionPrivacyPolicyScreen
import screens.introduction.IntroductionScreens

class IntroductionScreenTests : TestBase() {

    @RetryingTest(value = 3, name = "{displayName} - #{index}")
    @DisplayName("Verify introduction screens and privacy policy screen")
    @Tags(Tag("Smoke"), Tag("android"), Tag("iOS"))
    fun verifyIntroductionScreens() {
        IntroductionScreens.clickNextButton()
        IntroductionScreens.clickNextButton()
        IntroductionScreens.clickNextButton()
        IntroductionScreens.clickPrivacyPolicyButton()

        assertTrue(
            IntroductionPrivacyPolicyScreen.verifyPlaceholderTextIsVisible(),
            "placeholder text is not visible"
        )
    }

    @RetryingTest(value = 3, name = "{displayName} - #{index}")
    @DisplayName("Verify  introduction screens")
    @Tags(Tag("Smoke"), Tag("android"), Tag("iOS"))
    fun verifyNextIntroductionScreens() {
        IntroductionScreens.clickChangeLanguageButton()
        IntroductionChangeLanguageScreen.selectEnglishLanguageOption()
        IntroductionChangeLanguageScreen.clickBackButton()

        assertTrue(IntroductionScreens.verifyWelcomeTextIsVisible(), "welcome text is not visible")
    }
}