package feature.permissions

import domain.Platform
import helper.TestBase
import navigator.MenuNavigator
import navigator.OnboardingNavigator
import navigator.screen.MenuNavigatorScreen
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.dashboard.DashboardScreen
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
        resetIosPermissions()
        startDriver(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        menuScreen = MenuScreen()
        settingsScreen = SettingsScreen()
        notificationsScreen = NotificationsScreen()
        nativePermissionDialog = NativePermissionDialog()
    }

    @RetryingTest(value = MAX_SIMULATOR_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC88 Denying notification permission keeps notifications disabled")
    fun verifyDenyingNotificationPermissionKeepsNotificationsDisabled(testInfo: TestInfo) {
        setUp(testInfo)
        menuScreen.clickSettingsButton()
        settingsScreen.clickNotificationsButton()
        notificationsScreen.toggleNotifications()
        nativePermissionDialog.deny()
        if (notificationsScreen.platform() == Platform.ANDROID) {
            notificationsScreen.toggleNotifications()
            nativePermissionDialog.denyDontAskAgain()
        }
        assertTrue(!notificationsScreen.notificationsToggled(), "notifications are toggled on")
    }

    @RetryingTest(value = MAX_SIMULATOR_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("Notification permission drawer is shown once, on the second dashboard visit")
    fun verifyNotificationPermissionDrawerIsShownOnce(testInfo: TestInfo) {
        resetIosPermissions()
        startDriver(testInfo)
        val onboardingNavigator = OnboardingNavigator()
        val dashboardScreen = DashboardScreen()
        onboardingNavigator.toScreen(OnboardingNavigatorScreen.Dashboard, dismissNotificationsDrawer = false)
        assertFalse(dashboardScreen.notificationsDrawerVisible(), "notification permission drawer is visible on the first dashboard visit")

        onboardingNavigator.revisitDashboard()
        assertTrue(
            dashboardScreen.notificationsDrawerVisible(OnboardingNavigator.NOTIFICATIONS_DRAWER_TIMEOUT_SECONDS),
            "notification permission drawer is not visible on the second dashboard visit",
        )
        dashboardScreen.dismissNotificationsDrawer()

        onboardingNavigator.revisitDashboard()
        assertFalse(
            dashboardScreen.notificationsDrawerVisible(OnboardingNavigator.NOTIFICATIONS_DRAWER_TIMEOUT_SECONDS),
            "notification permission drawer is visible again after it was dismissed",
        )
    }
}
