package feature.menu

import helper.TestBase
import navigator.MenuNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.history.HistoryOverviewScreen
import screen.menu.MenuScreen
import screen.settings.SettingsScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${MenuTests.USE_CASE} Show app menu [${MenuTests.JIRA_ID}]")
class MenuTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 9.1"
        const val JIRA_ID = "PVW-1225"
    }

    private lateinit var menuScreen: MenuScreen

    fun setUp() {
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)

        menuScreen = MenuScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1 The app menu is accessible from the footer menu. [$JIRA_ID]")
    fun verifyMenuScreen() {
        setUp()
        assertTrue(menuScreen.visible(), "menu screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.2 The app menu offers an option to log out. [$JIRA_ID]")
    fun verifyLogoutButtonVisible() {
        setUp()
        assertTrue(menuScreen.logoutButtonVisible(), "logout button is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.3 The app menu contains the following items: Help, History, Settings, Feedback, About. [$JIRA_ID]")
    fun verifyMenuItemsVisible() {
        setUp()
        assertTrue(menuScreen.menuListButtonsVisible(), "menu list buttons are not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.4 The settings menu contains the following items: Change pin, Setup biometrics, Change language, Clear data. [$JIRA_ID]")
    fun verifySettingsItemsVisible() {
        setUp()
        menuScreen.clickSettingsButton()

        val settingsScreen = SettingsScreen()
        assertTrue(settingsScreen.settingsButtonsVisible(), "settings buttons are not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.5 All items opened from the menu offer a back button returning the user to the main menu. [$JIRA_ID]")
    fun verifyBackButtons() {
        setUp()
        // Navigate to help and back
        menuScreen.clickHelpButton()
        menuScreen.clickBottomBackButton()

        // Navigate to history overview and back
        menuScreen.clickHistoryButton()
        val historyOverviewScreen = HistoryOverviewScreen()
        historyOverviewScreen.clickBottomBackButton()

        // Navigate to settings and back
        menuScreen.clickSettingsButton()
        menuScreen.clickBottomBackButton()

        // Navigate to feedback and back
        menuScreen.clickFeedbackButton()
        menuScreen.clickBottomBackButton()

        // Navigate to about and back
        menuScreen.clickAboutButton()
        menuScreen.clickBottomBackButton()

        assertTrue(menuScreen.visible(), "menu screen is not visible")
    }
}
