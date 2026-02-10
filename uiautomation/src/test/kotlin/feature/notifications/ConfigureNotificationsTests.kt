package feature.notifications

import helper.LocalizationHelper
import helper.TasDataHelper
import helper.TestBase
import navigator.MenuNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.dashboard.DashboardScreen
import screen.menu.MenuScreen
import screen.security.PinScreen
import screen.settings.NotificationsDebugScreen
import screen.settings.NotificationsDebugScreen.CardNotificationType.EXPIRED
import screen.settings.NotificationsScreen
import screen.settings.SettingsScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("UC 9.11 Configure Notifications")
class ConfigureNotificationsTests : TestBase() {

    private lateinit var l10n: LocalizationHelper
    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var menuScreen: MenuScreen
    private lateinit var settingsScreen: SettingsScreen
    private lateinit var notificationsScreen: NotificationsScreen
    private lateinit var notificationsDebugScreen: NotificationsDebugScreen
    private lateinit var pinScreen: PinScreen
    private lateinit var tasData: TasDataHelper

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        l10n = LocalizationHelper()
        dashboardScreen = DashboardScreen()
        menuScreen = MenuScreen()
        settingsScreen = SettingsScreen()
        notificationsScreen = NotificationsScreen()
        notificationsDebugScreen = NotificationsDebugScreen()
        pinScreen = PinScreen()
        tasData = TasDataHelper()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC72 turn on notifications after these were initially turned of")
    fun verifyNotificationsConfiguration(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)

        menuScreen.clickSettingsButton()
        settingsScreen.clickNotificationsButton()
        notificationsScreen.clickDebugScreenButton()
        notificationsDebugScreen.openPendingTab()
        assertTrue(!notificationsDebugScreen.isNotificationVisible(tasData.getPidDisplayName(), EXPIRED), "notification scheduled while it should not")

        notificationsDebugScreen.clickBackButton()
        notificationsScreen.toggleNotifications()
        notificationsScreen.clickDebugScreenButton()
        notificationsDebugScreen.openPendingTab()
        assertTrue(notificationsDebugScreen.isNotificationVisible(tasData.getPidDisplayName(), EXPIRED), "notification not scheduled")
    }
}
