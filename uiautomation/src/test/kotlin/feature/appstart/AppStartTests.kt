package feature.appstart

import helper.TestBase
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Nested
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junitpioneer.jupiter.RetryingTest
import screen.introduction.IntroductionScreen

@DisplayName("${AppStartTests.USE_CASE} Open the App [${AppStartTests.JIRA_ID}]")
class AppStartTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 1.2"
        const val JIRA_ID = "PVW-1223"
    }

    private lateinit var introductionScreen: IntroductionScreen

    @BeforeEach
    fun setUp() {
        introductionScreen = IntroductionScreen()
    }

    /**
     * 1. When the App is started, it shows a loading screen until necessary resources are loaded, including the name and logo of the app.
     * >> Setup and initial loading is so fast, that the Flutter splash screen is currently not visible.
     */

    @Nested
    @DisplayName("$USE_CASE.2 When a language has not been configured in-app, the App uses the language preferences of the OS. [${JIRA_ID}]")
    inner class LanguagePreferences {

        @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
        @Tags(Tag("dutch"))
        @DisplayName("$USE_CASE.2.1 If the device language is set to Dutch, then the app starts in Dutch. [${JIRA_ID}]")
        fun verifyDutchLanguage() {
            assertTrue(introductionScreen.nextButtonTextVisible("Volgende"))
        }

        @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
        @Tags(Tag("english"))
        @DisplayName("$USE_CASE.2.2 If the device language is set to English, then the app starts in English. [${JIRA_ID}]")
        fun verifyEnglishLanguage() {
            assertTrue(introductionScreen.nextButtonTextVisible("Next"))
        }

        @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
        @Tags(Tag("france"))
        @DisplayName("$USE_CASE.2.3 If the device language is set to France, then the app starts in English. [${JIRA_ID}]")
        fun verifyOtherLanguage() {
            assertTrue(introductionScreen.nextButtonTextVisible("Next"))
        }
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.3 When a PIN has not yet been set up, the App starts UC 1.1 Introduce the App. [${JIRA_ID}]")
    fun verifyAppStartBeforeSecuritySetup() {
        assertTrue(introductionScreen.page1Visible())
    }

    /**
     * 4. When a PIN has been set up before, the App starts UC 2.3 Unlock the App.
     * >> Should be tested in security related feature.
     */
}
