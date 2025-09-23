package nativefeature.menu_and_settings

import helper.TestBase
import nativenavigator.MenuNavigator
import nativenavigator.screen.MenuNavigatorScreen
import nativescreen.menu.MenuScreen
import nativescreen.settings.ChangeLanguageScreen
import nativescreen.settings.SettingsScreen
import org.junit.jupiter.api.Assertions.assertAll
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("UC9.3 Change app language")
class ChangeLanguageTests : TestBase() {

    private lateinit var changeLanguageScreen: ChangeLanguageScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickSettingsButton()
        SettingsScreen().clickChangeLanguageButton()
        changeLanguageScreen = ChangeLanguageScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @Tags(Tag("english"))
    @DisplayName("LTC38, LTC39 Select a new language, English")
    fun verifyDutchLanguageSelect(testInfo: TestInfo) {
        setUp(testInfo)
        assertAll(
            { assertTrue(changeLanguageScreen.visible(), "change language screen is not visible") },
            { assertTrue(changeLanguageScreen.languageButtonsVisible(), "language buttons are not visible") },
            { assertTrue(changeLanguageScreen.englishScreenTitleVisible(), "english screen title is not visible") }
        )

        changeLanguageScreen.clickDutchButton()
        assertTrue(changeLanguageScreen.dutchScreenTitleVisible(), "dutch screen title is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @Tags(Tag("dutch"), Tag("nativesmoke"))
    @DisplayName("LTC38, LTC39 Select a new language, Dutch")
    fun verifyEnglishLanguageSelect(testInfo: TestInfo) {
        setUp(testInfo)
        assertAll(
            { assertTrue(changeLanguageScreen.visible(), "change language screen is not visible") },
            { assertTrue(changeLanguageScreen.languageButtonsVisible(), "language buttons are not visible") },
            { assertTrue(changeLanguageScreen.dutchScreenTitleVisible(), "dutch screen title is not visible") }
        )

        changeLanguageScreen.clickEnglishButton()
        assertTrue(changeLanguageScreen.englishScreenTitleVisible(), "english screen title is not visible")
    }
}

