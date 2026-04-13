package feature.permissions

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.dashboard.DashboardScreen
import screen.disclosure.QRScanner
import screen.permissions.NativePermissionDialog
import screen.security.PinScreen
import util.MobileActions

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("Camera Permission")
class CameraPermissionTests : TestBase() {

    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var nativePermissionDialog: NativePermissionDialog
    private lateinit var qrScanner: QRScanner
    private lateinit var pinScreen: PinScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.Dashboard)
        dashboardScreen = DashboardScreen()
        nativePermissionDialog = NativePermissionDialog()
        qrScanner = QRScanner()
        pinScreen = PinScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("Camera permission not granted shows in-app permission screen")
    fun verifyCameraPermissionScreenShownWhenPermissionNotGranted(testInfo: TestInfo) {
        setUp(testInfo)
        dashboardScreen.openQRScanner()
        when (dashboardScreen.platformName()) {
            "ANDROID" -> nativePermissionDialog.deny()
            "IOS" -> nativePermissionDialog.deny()
        }
        assertTrue(qrScanner.permissionHintVisible(), "camera permission screen is not visible")
        assertTrue(qrScanner.grantPermissionButtonVisible(), "grant camera permission button is not visible")
    }

    @Tags(Tag("androidOnly"))
    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("Camera permission granted once is re-requested after app restart")
    fun verifyCameraPermissionRequestedAgainAfterOnceGrantAndAppRestart(testInfo: TestInfo) {
        setUp(testInfo)
        dashboardScreen.openQRScanner()
        nativePermissionDialog.allowOneTimeOnly()
        assertTrue(qrScanner.visible(), "QR scanner is not visible after granting camera permission once")
        qrScanner.goBack()

        dashboardScreen.closeApp()
        Thread.sleep(MobileActions.DEFAULT_RESET_SLEEP)
        dashboardScreen.openApp()
        if (pinScreen.pinScreenVisible()) {
            pinScreen.enterPin(DEFAULT_PIN)
        }

        dashboardScreen.openQRScanner()
        assertTrue(nativePermissionDialog.visible(), "camera permission is not re-requested after app restart")
    }
}
