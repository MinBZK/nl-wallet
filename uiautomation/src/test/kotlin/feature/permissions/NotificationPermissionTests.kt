package feature.permissions

import helper.TestBase
import navigator.MenuNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.menu.MenuScreen
import screen.permissions.NativePermissionDialog
import screen.settings.NotificationsScreen
import screen.settings.SettingsScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("Notification Permission")
class NotificationPermissionTests : TestBase() {

    private lateinit var menuScreen: MenuScreen
    private lateinit var settingsScreen: SettingsScreen
    private lateinit var notificationsScreen: NotificationsScreen
    private lateinit var nativePermissionDialog: NativePermissionDialog

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        menuScreen = MenuScreen()
        settingsScreen = SettingsScreen()
        notificationsScreen = NotificationsScreen()
        nativePermissionDialog = NativePermissionDialog()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("Denying notification permission keeps notifications disabled")
    fun verifyDenyingNotificationPermissionKeepsNotificationsDisabled(testInfo: TestInfo) {
        setUp(testInfo)
        menuScreen.clickSettingsButton()
        settingsScreen.clickNotificationsButton()
        notificationsScreen.toggleNotifications()
        nativePermissionDialog.deny()
        when (notificationsScreen.platformName()) {
            "ANDROID" -> {
                notificationsScreen.toggleNotifications()
                nativePermissionDialog.denyDontAskAgain()
            }
        }
        assertTrue(!notificationsScreen.notificationsToggled(), "notifications are toggled on")
    }
}
