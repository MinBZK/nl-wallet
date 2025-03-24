package feature.settings

import helper.TestBase
import navigator.MenuNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertAll
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Nested
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.menu.MenuScreen
import screen.settings.ChangeLanguageScreen
import screen.settings.SettingsScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${ChangeLanguageTests.USE_CASE} User changes language [${ChangeLanguageTests.JIRA_ID}]")
class ChangeLanguageTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 9.3"
        const val JIRA_ID = "PVW-1224"
    }

    private lateinit var changeLanguageScreen: ChangeLanguageScreen

    fun setUp() {
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)

        MenuScreen().clickSettingsButton()
        SettingsScreen().clickChangeLanguageButton()

        changeLanguageScreen = ChangeLanguageScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1 App settings menu displays option to change language. 2 Language screen offers two options: English & Dutch. [$JIRA_ID]")
    fun verifyChangeLanguageScreen() {
        setUp()
        assertAll(
            { assertTrue(changeLanguageScreen.visible(), "change language screen is not visible") },
            { assertTrue(changeLanguageScreen.languageButtonsVisible(), "language buttons are not visible") }
        )
    }

    @Nested
    @DisplayName("$USE_CASE.3 When the User selects a language, the app immediately uses the newly selected language. [$JIRA_ID]")
    inner class LanguageChange {

        @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
        @Tags(Tag("english"))
        @DisplayName("$USE_CASE.3.1 When the User selects Dutch, the app immediately uses Dutch. [$JIRA_ID]")
        fun verifyDutchLanguageSelect() {
            setUp()
            assertTrue(changeLanguageScreen.englishScreenTitleVisible(), "english screen title is not visible")
            changeLanguageScreen.clickDutchButton()

            assertTrue(changeLanguageScreen.dutchScreenTitleVisible(), "dutch screen title is not visible")
        }

        @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
        @Tags(Tag("dutch"), Tag("smoke"))
        @DisplayName("$USE_CASE.3.2 When the User selects English, the app immediately uses English. [$JIRA_ID]")
        fun verifyEnglishLanguageSelect() {
            setUp()
            assertTrue(changeLanguageScreen.dutchScreenTitleVisible(), "dutch screen title is not visible")
            changeLanguageScreen.clickEnglishButton()

            assertTrue(changeLanguageScreen.englishScreenTitleVisible(), "english screen title is not visible")
        }
    }
}
