package feature.permissions

import data.TestConfigRepository.Companion.testConfig
import domain.Platform
import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Assumptions.assumeTrue
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
        // The iOS simulator has no Bluetooth and on real iOS devices the permissions cannot be reliably reset.
        assumeTrue(testConfig.platform == Platform.ANDROID, "Bluetooth permission prompt is not available on the iOS simulator")
        setUp(testInfo)
        dashboardScreen.showQRCode()
        when (dashboardScreen.platform()) {
            Platform.ANDROID -> {
                nativePermissionDialog.deny()
                dashboardScreen.showQRCode()
                nativePermissionDialog.denyDontAskAgain()
            }
            Platform.IOS -> {
                nativePermissionDialog.deny()
            }
        }
        assertTrue(bluetoothPermissionScreen.visible(), "bluetooth permission screen is not visible")
        assertTrue(bluetoothPermissionScreen.descriptionVisible(), "bluetooth permission description is not visible")
        assertTrue(bluetoothPermissionScreen.openSettingsButtonVisible(), "open settings button is not visible")
    }
}
