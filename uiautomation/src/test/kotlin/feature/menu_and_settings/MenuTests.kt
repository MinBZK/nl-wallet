package feature.menu_and_settings

import helper.TestBase
import navigator.MenuNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertAll
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.history.HistoryOverviewScreen
import screen.menu.MenuScreen
import screen.settings.SettingsScreen
import util.MobileActions.Companion.SCREEN_TRANSITION_MILLIS

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("UC 9.1 Show app menu")
class MenuTests : TestBase() {

    private lateinit var menuScreen: MenuScreen
    private lateinit var historyOverviewScreen: HistoryOverviewScreen
    private lateinit var settingsScreen: SettingsScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)

        menuScreen = MenuScreen()
        historyOverviewScreen = HistoryOverviewScreen()
        settingsScreen = SettingsScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC26 LTC27 Show app & Settings menu")
    @Tags(Tag("a11yBatch2"))
    fun verifyMenuScreen(testInfo: TestInfo) {
        setUp(testInfo)
        assertTrue(menuScreen.menuListButtonsVisible(), "menu screen is not visible")

        menuScreen.clickHelpAndInfoButton()
        Thread.sleep(SCREEN_TRANSITION_MILLIS)
        menuScreen.clickBottomBackButton()
        menuScreen.clickHistoryButton()

        Thread.sleep(SCREEN_TRANSITION_MILLIS)
        historyOverviewScreen.clickBottomBackButton()
        menuScreen.clickSettingsButton()
        assertTrue(settingsScreen.settingsButtonsVisible(), "settings buttons are not visible")

        Thread.sleep(SCREEN_TRANSITION_MILLIS)
        menuScreen.clickBottomBackButton()
        menuScreen.clickFeedbackButton()

        Thread.sleep(SCREEN_TRANSITION_MILLIS)
        menuScreen.clickBottomBackButton()

        menuScreen.clickAboutButton()
        Thread.sleep(SCREEN_TRANSITION_MILLIS)
        menuScreen.clickBottomBackButton()

        assertAll(
            { assertTrue(menuScreen.menuListButtonsVisible(), "menu list buttons are not visible") },
            { assertTrue(menuScreen.logoutButtonVisible(), "logout button is notvisible") },
        )
    }
}
