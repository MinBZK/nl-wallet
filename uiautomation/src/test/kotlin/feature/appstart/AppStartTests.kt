package feature.appstart

import helper.TestBase
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Nested
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.Test
import screen.introduction.IntroductionScreen

@DisplayName("UC 1.2 - Open the App [PVW-1223]")
class AppStartTests : TestBase() {

    private lateinit var introductionScreen: IntroductionScreen

    @BeforeEach
    fun setUp() {
        introductionScreen = IntroductionScreen()
    }

    @Test
    @DisplayName("1. When the App is started, it shows a loading screen until necessary resources are loaded, including the name and logo of the app.")
    fun verifySplashScreen() {
        //FUTURE: Until the splash screen has a minimum display duration; this is not testable in a stable way, the future implementation would look like this:
        // assertTrue(splashScreen.visible(), "Splash screen is not visible")
        // assertEquals(splashScreen.readAppNameText(), l10n.getString("appTitle"), "App name is not equal to expected value")
    }

    @Nested
    @DisplayName("2. When a language has not been configured in-app, the App uses the language preferences of the OS.")
    inner class LanguagePreferences {

        @Test
        @Tags(Tag("dutch"))
        @DisplayName("2.1. If the device language is set to Dutch, then the app starts in Dutch")
        fun verifyDutchLanguage() {
            assertEquals(introductionScreen.readNextButtonText(), "Volgende")
        }

        @Test
        @Tags(Tag("english"))
        @DisplayName("2.2. If the device language is set to English, then the app starts in English")
        fun verifyEnglishLanguage() {
            assertEquals(introductionScreen.readNextButtonText(), "Next")
        }

        @Test
        @Tags(Tag("france"))
        @DisplayName("2.3. If the device language is set to France, then the app starts in English")
        fun verifyOtherLanguage() {
            assertEquals(introductionScreen.readNextButtonText(), "Next")
        }
    }

    @Test
    @DisplayName("3. When a PIN has not yet been set up, the App starts UC 1.1 Introduce the App")
    fun verifyAppStartBeforeSecuritySetup() {
        assertTrue(introductionScreen.page1Visible())
    }

    //@Test
    @DisplayName("4. When a PIN has been set up before, the App starts UC 2.3 Unlock the App")
    fun verifyAppStartAfterSecuritySetup() {
        // Should be tested in security related feature.
    }
}
