package nativefeature.disclosure

import helper.TestBase
import nativenavigator.MenuNavigator
import nativenavigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import nativescreen.dashboard.DashboardScreen
import nativescreen.disclosure.QRScanner


@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("UC9.9 User scans QR")
class QRScannerTests : TestBase() {

    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var qrScanner: QRScanner

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC7 QR scanner")
    fun verifyScanInAppDialog(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Dashboard)
        dashboardScreen = DashboardScreen()
        dashboardScreen.openQRScanner()
        qrScanner = QRScanner()
        assertTrue(
            qrScanner.visible(),
            "QR Scanner is not visible"
        )
        qrScanner.enableTorch()
        qrScanner.disableTorch()
        assertTrue(
            qrScanner.visible(),
            "QR Scanner is not visible")
        qrScanner.goBack()
        assertTrue(
            dashboardScreen.visible(),
            "dashboard is not visible"
        )
    }
}
