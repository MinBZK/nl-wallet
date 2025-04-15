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
import screen.disclosure.ScanWithWalletDialog
import screen.introduction.IntroductionScreen
import java.net.URLEncoder

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${UniversalLinkTests.USE_CASE} Show app menu [${UniversalLinkTests.JIRA_ID}]")
class UniversalLinkTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 5.1"
        const val JIRA_ID = "PVW-1347"
    }

    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var scanWithWalletDialog: ScanWithWalletDialog
    private lateinit var introductionScreen: IntroductionScreen
    private lateinit var expiredUniversalLinkFromCameraApp: String



    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        expiredUniversalLinkFromCameraApp = "https://app.example.com/deeplink/disclosure?" + mapOf(
            "request_uri" to "https://example.com/disclosure/sessions/CYqJdDLRIkFArxoWLXLUYaAkUiK4A6YF/request_uri?session_type=cross_device&ephemeral_id=02a1bf4d24a54228be1ba88576bfd4d7df8759d23df90822fda8f49da6826213&time=2025-04-10T10%3A44%3A15.629765875Z",
            "request_uri_method" to "post",
            "client_id" to "mijn_amsterdam.example.com",
        ).map { "${it.key}=${URLEncoder.encode(it.value, Charsets.UTF_8)}" }.joinToString("&")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.When the App receives a universal link that is the result of scanning a QR (given a list of known QR-scanner/camera apps), the App prompts the user to rescan the QR with the in-app scanner, for security reasons, and offers them the option to open the scanner.[$JIRA_ID]")
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
    @DisplayName("$USE_CASE.When the App is installed but not activated, the app shows the onboarding procedure (a message is future work).[$JIRA_ID]")
    fun verifyWhenAppNotActivated(testInfo: TestInfo) {
        setUp(testInfo)
        introductionScreen = IntroductionScreen()
        introductionScreen.openUniversalLink(expiredUniversalLinkFromCameraApp)
        assertTrue(introductionScreen.page1Visible(), "introduction screen is  not visible")
    }
}
