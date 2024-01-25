package feature.menu

import helper.TestBase
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Test
import screen.dashboard.DashboardScreen
import screen.digid.DigidLoginMockWebPage
import screen.digid.DigidLoginStartWebPage
import screen.introduction.IntroductionConditionsScreen
import screen.introduction.IntroductionExpectationsScreen
import screen.introduction.IntroductionPrivacyScreen
import screen.introduction.IntroductionScreen
import screen.menu.MenuScreen
import screen.personalize.PersonalizeInformScreen
import screen.personalize.PersonalizePidPreviewScreen
import screen.personalize.PersonalizeSuccessScreen
import screen.security.PinScreen
import screen.security.SetupSecurityCompletedScreen
import screen.settings.SettingsScreen

@DisplayName("UC 9.1 - Show app menu [PVW-1225]")
class MenuTests : TestBase() {

    private val chosenPin = "122222"

    private lateinit var menuScreen: MenuScreen

    @BeforeEach
    fun setUp() {
        val introductionScreen = IntroductionScreen()
        val expectationsScreen = IntroductionExpectationsScreen()
        val privacyScreen = IntroductionPrivacyScreen()
        val conditionsScreen = IntroductionConditionsScreen()
        val pinScreen = PinScreen()
        val setupSecurityCompletedScreen = SetupSecurityCompletedScreen()
        val personalizeInformScreen = PersonalizeInformScreen()
        val digidLoginStartWebPage = DigidLoginStartWebPage()
        val digidLoginMockWebPage = DigidLoginMockWebPage()
        val personalizePidPreviewScreen = PersonalizePidPreviewScreen()
        val personalizeSuccessScreen = PersonalizeSuccessScreen()
        val dashboardScreen = DashboardScreen()

        // Start all tests on digid login start web page
        introductionScreen.clickSkipButton()
        expectationsScreen.clickNextButton()
        privacyScreen.clickNextButton()
        conditionsScreen.clickNextButton()
        pinScreen.enterPin(chosenPin)
        pinScreen.enterPin(chosenPin)
        setupSecurityCompletedScreen.clickNextButton()
        personalizeInformScreen.clickLoginWithDigidButton()
        personalizeInformScreen.switchToWebView()
        digidLoginStartWebPage.clickMockLoginButton()
        digidLoginMockWebPage.clickLoginButton()
        personalizePidPreviewScreen.switchToApp()
        personalizePidPreviewScreen.clickAcceptButton()
        pinScreen.enterPin(chosenPin)
        personalizeSuccessScreen.clickNextButton()
        dashboardScreen.clickMenuButton()

        menuScreen = MenuScreen()
    }

    @Test
    @DisplayName("1. The app menu is accessible from the footer menu.")
    fun verifyMenuScreen() {
        assertTrue(menuScreen.visible(), "menu screen is not visible")
    }

    @Test
    @DisplayName("2. The app menu offers an option to log out.")
    fun verifyLogoutButtonVisible() {
        assertTrue(menuScreen.logoutButtonVisible(), "logout button is not visible")
    }

    @Test
    @DisplayName("3. The app menu contains the following items: Help, History, Settings, Feedback, About.")
    fun verifyMenuItemsVisible() {
        assertTrue(menuScreen.menuListButtonsVisible(), "menu list buttons are not visible")
    }

    @Test
    @DisplayName("4. The settings menu contains the following items: Change pin, Setup biometrics, Change language, Clear data.")
    fun verifySettingsItemsVisible() {
        menuScreen.clickSettingsButton()

        val settingsScreen = SettingsScreen()
        assertTrue(settingsScreen.settingsButtonsVisible(), "settings buttons are not visible")
    }

    @Test
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
