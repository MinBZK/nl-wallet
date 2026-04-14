package feature.permissions

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.dashboard.DashboardScreen
import screen.permissions.BluetoothPermissionScreen
import screen.permissions.NativePermissionDialog

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("Bluetooth Permission")
class BluetoothPermissionTests : TestBase() {

    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var bluetoothPermissionScreen: BluetoothPermissionScreen
    private lateinit var nativePermissionDialog: NativePermissionDialog

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.Dashboard)
        dashboardScreen = DashboardScreen()
        bluetoothPermissionScreen = BluetoothPermissionScreen()
        nativePermissionDialog = NativePermissionDialog()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("Bluetooth permission not granted shows in-app permission screen")
    fun verifyBluetoothPermissionScreenShownWhenPermissionNotGranted(testInfo: TestInfo) {
        setUp(testInfo)
        dashboardScreen.showQRCode()
        when (dashboardScreen.platformName()) {
            "ANDROID" -> {
                nativePermissionDialog.deny()
                dashboardScreen.showQRCode()
                nativePermissionDialog.denyDontAskAgain()
            }
            "IOS" -> {
                nativePermissionDialog.deny()
            }
        }
        assertTrue(bluetoothPermissionScreen.visible(), "bluetooth permission screen is not visible")
        assertTrue(bluetoothPermissionScreen.descriptionVisible(), "bluetooth permission description is not visible")
        assertTrue(bluetoothPermissionScreen.openSettingsButtonVisible(), "open settings button is not visible")
    }
}
