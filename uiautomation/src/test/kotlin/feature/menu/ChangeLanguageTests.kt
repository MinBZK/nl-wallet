package feature.menu

import helper.TestBase
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Nested
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junitpioneer.jupiter.RetryingTest
import screen.change_language.ChangeLanguageScreen
import screen.dashboard.DashboardScreen
import screen.menu.MenuScreen
import screen.settings.SettingsScreen
import setup.OnboardingNavigator
import setup.Screen

@DisplayName("UC 9.3 - User changes language [PVW-1224]")
class ChangeLanguageTests : TestBase() {

    private val onboardingNavigator = OnboardingNavigator()

    private lateinit var changeLanguageScreen: ChangeLanguageScreen

    @BeforeEach
    fun setUp() {
        onboardingNavigator.toScreen(Screen.Dashboard)

        DashboardScreen().clickMenuButton()
        MenuScreen().clickSettingsButton()
        SettingsScreen().clickChangeLanguageButton()

        changeLanguageScreen = ChangeLanguageScreen()
    }

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("1. App settings menu displays option to change language.")
    fun verifyChangeLanguageScreen() {
        assertTrue(changeLanguageScreen.visible(), "change language screen is not visible")
    }

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("2. Language screen offers two options: English & Dutch.")
    fun verifyLanguageButtonsVisible() {
        assertTrue(changeLanguageScreen.languageButtonsVisible(), "language buttons are not visible")
    }

    @Nested
    @DisplayName("3. When the User selects a language, the app immediately uses the newly selected language.")
    inner class LanguageChange {

        @RetryingTest(MAX_RETRY_COUNT)
        @Tags(Tag("english"))
        @DisplayName("3.1. When the User selects Dutch, the app immediately uses Dutch.")
        fun verifyDutchLanguageSelect() {
            assertTrue(changeLanguageScreen.englishScreenTitleVisible(), "english screen title is not visible")
            changeLanguageScreen.clickDutchButton()

            assertTrue(changeLanguageScreen.dutchScreenTitleVisible(), "dutch screen title is not visible")
        }

        @RetryingTest(MAX_RETRY_COUNT)
        @Tags(Tag("dutch"))
        @DisplayName("3.2. When the User selects English, the app immediately uses English.")
        fun verifyEnglishLanguageSelect() {
            assertTrue(changeLanguageScreen.dutchScreenTitleVisible(), "dutch screen title is not visible")
            changeLanguageScreen.clickEnglishButton()

            assertTrue(changeLanguageScreen.englishScreenTitleVisible(), "english screen title is not visible")
        }
    }
}
