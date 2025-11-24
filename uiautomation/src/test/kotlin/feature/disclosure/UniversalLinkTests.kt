package feature.disclosure

import helper.TestBase
import navigator.MenuNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertAll
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.dashboard.DashboardScreen
import screen.demo.DemoScreen
import screen.disclosure.ScanWithWalletDialog
import java.net.URLEncoder

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("Partial Flow 2.7 Resolve a universal link")
class UniversalLinkTests : TestBase() {

    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var scanWithWalletDialog: ScanWithWalletDialog
    private lateinit var expiredUniversalLinkFromCameraApp: String
    private lateinit var demoScreen: DemoScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        expiredUniversalLinkFromCameraApp = "https://app.example.com/deeplink/disclosure?" + mapOf(
            "request_uri" to "https://example.com/disclosure/sessions/CYqJdDLRIkFArxoWLXLUYaAkUiK4A6YF/request_uri?session_type=cross_device&ephemeral_id=02a1bf4d24a54228be1ba88576bfd4d7df8759d23df90822fda8f49da6826213&time=2025-04-10T10%3A44%3A15.629765875Z",
            "request_uri_method" to "post",
            "client_id" to "mijn_amsterdam.example.com",
        ).map { "${it.key}=${URLEncoder.encode(it.value, Charsets.UTF_8)}" }.joinToString("&")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC36 Universal link is opened via external QR scanner")
    fun verifyScanInAppDialog(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        dashboardScreen = DashboardScreen()
        dashboardScreen.openUniversalLink(expiredUniversalLinkFromCameraApp)
        scanWithWalletDialog = ScanWithWalletDialog()
        assertAll(
            { assertTrue(scanWithWalletDialog.visible(), "scan with wallet dialog is not visible") },
            { assertTrue(scanWithWalletDialog.scanWithWalletDialogBodyVisible(), "scan with wallet dialog subtitle is not visible") },
            { assertTrue(scanWithWalletDialog.scanWithWalletButtonVisible(), "scan with wallet button is not visible") },
        )
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC44 Wallet not created when universal link is invoked")
    fun verifyWhenAppNotActivated(testInfo: TestInfo) {
        setUp(testInfo)
        demoScreen = DemoScreen()
        demoScreen.openUniversalLink(expiredUniversalLinkFromCameraApp)
        assertTrue(demoScreen.visible(), "demo screen is not visible")
    }
}
