package feature.menu

import helper.TestBase
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junitpioneer.jupiter.RetryingTest
import screen.dashboard.DashboardScreen
import screen.menu.MenuScreen
import screen.settings.SettingsScreen
import setup.OnboardingNavigator
import setup.Screen

@DisplayName("UC 9.1 - Show app menu [PVW-1225]")
class MenuTests : TestBase() {

    private val onboardingNavigator = OnboardingNavigator()

    private lateinit var menuScreen: MenuScreen

    @BeforeEach
    fun setUp() {
        onboardingNavigator.toScreen(Screen.Dashboard)

        DashboardScreen().clickMenuButton()

        menuScreen = MenuScreen()
    }

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("1. The app menu is accessible from the footer menu.")
    fun verifyMenuScreen() {
        assertTrue(menuScreen.visible(), "menu screen is not visible")
    }

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("2. The app menu offers an option to log out.")
    fun verifyLogoutButtonVisible() {
        assertTrue(menuScreen.logoutButtonVisible(), "logout button is not visible")
    }

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("3. The app menu contains the following items: Help, History, Settings, Feedback, About.")
    fun verifyMenuItemsVisible() {
        assertTrue(menuScreen.menuListButtonsVisible(), "menu list buttons are not visible")
    }

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("4. The settings menu contains the following items: Change pin, Setup biometrics, Change language, Clear data.")
    fun verifySettingsItemsVisible() {
        menuScreen.clickSettingsButton()

        val settingsScreen = SettingsScreen()
        assertTrue(settingsScreen.settingsButtonsVisible(), "settings buttons are not visible")
    }

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("5. All items opened from the menu offer a back button returning the user to the main menu.")
    fun verifyBackButtons() {
        menuScreen.clickHelpButton()
        menuScreen.clickBackButton()

        menuScreen.clickHistoryButton()
        menuScreen.clickBackButton()

        menuScreen.clickSettingsButton()
        menuScreen.clickBackButton()

        menuScreen.clickFeedbackButton()
        menuScreen.clickBackButton()

        menuScreen.clickAboutButton()
        menuScreen.clickBackButton()

        assertTrue(menuScreen.visible(), "menu screen is not visible")
    }
}
