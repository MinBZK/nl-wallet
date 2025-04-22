package feature.disclosure

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
import screen.disclosure.QRScanner


@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${QRScannerTests.USE_CASE} User scans QR for presentation [${QRScannerTests.JIRA_ID}]")
class QRScannerTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 5.1"
        const val JIRA_ID = "PVW-1346"
    }

    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var qrScanner: QRScanner

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1.The App offers a QR scanner. 2.The QR scanner has a button to toggle the device flashlight, off by default. 3.the QR scanner reports periodically: no QR has been detected yet[$JIRA_ID]")
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
