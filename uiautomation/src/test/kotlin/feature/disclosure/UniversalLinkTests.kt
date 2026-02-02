package feature.disclosure

import helper.OrganizationAuthMetadataHelper
import helper.OrganizationAuthMetadataHelper.Organization.INSURANCE
import helper.TestBase
import navigator.MenuNavigator
import navigator.OnboardingNavigator
import navigator.screen.MenuNavigatorScreen
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junit.jupiter.api.assertAll
import org.junitpioneer.jupiter.RetryingTest
import screen.dashboard.DashboardScreen
import screen.demo.DemoScreen
import screen.disclosure.ScanWithWalletDialog
import screen.error.InvalidIssuanceULErrorScreen
import screen.issuance.DisclosureIssuanceScreen
import screen.issuance.FinishWalletDialog
import screen.security.PinScreen
import java.net.URLEncoder

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("Partial Flow 2.7 Resolve a universal link")
class UniversalLinkTests : TestBase() {

    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var scanWithWalletDialog: ScanWithWalletDialog
    private lateinit var expiredDisclosureUniversalLinkFromCameraApp: String
    private lateinit var demoScreen: DemoScreen
    private lateinit var invalidIssuanceULErrorScreen: InvalidIssuanceULErrorScreen
    private lateinit var disclosureForIssuanceScreen: DisclosureIssuanceScreen
    private lateinit var organizationAuthMetadata: OrganizationAuthMetadataHelper
    private lateinit var pinScreen: PinScreen
    private lateinit var finishWalletDialog: FinishWalletDialog

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        expiredDisclosureUniversalLinkFromCameraApp = "https://app.example.com/deeplink/disclosure?" + mapOf(
            "request_uri" to "https://example.com/disclosure/sessions/CYqJdDLRIkFArxoWLXLUYaAkUiK4A6YF/request_uri?session_type=cross_device&ephemeral_id=02a1bf4d24a54228be1ba88576bfd4d7df8759d23df90822fda8f49da6826213&time=2025-04-10T10%3A44%3A15.629765875Z",
            "request_uri_method" to "post",
            "client_id" to "mijn_amsterdam.example.com",
        ).map { "${it.key}=${URLEncoder.encode(it.value, Charsets.UTF_8)}" }.joinToString("&")
        dashboardScreen = DashboardScreen()
        demoScreen = DemoScreen()
        invalidIssuanceULErrorScreen = InvalidIssuanceULErrorScreen()
        scanWithWalletDialog = ScanWithWalletDialog()
        organizationAuthMetadata = OrganizationAuthMetadataHelper()
        disclosureForIssuanceScreen = DisclosureIssuanceScreen()
        pinScreen = PinScreen()
        finishWalletDialog = FinishWalletDialog()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC33 Open app via universal link")
    fun verifyUlOpensApp(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Dashboard)
        val issuanceUniversalLink = "https://app.example.com/deeplink/disclosure_based_issuance?request_uri=https%3A%2F%2Fexample.com%2Fcd96997cf3772b54a9a0c9f2d261a401%2Fdisclosure%2Finsurance%2Frequest_uri%3Fsession_type%3Dsame_device&request_uri_method=post&client_id=insurance.example.com"
        dashboardScreen.closeApp()
        dashboardScreen.openUniversalLink(issuanceUniversalLink)
        pinScreen.enterPin(DEFAULT_PIN)

        assertTrue(disclosureForIssuanceScreen.organizationNameVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", INSURANCE)))

    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC36 Universal link is opened via external QR scanner")
    fun verifyScanInAppDialog(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        dashboardScreen.openUniversalLink(expiredDisclosureUniversalLinkFromCameraApp)
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
        demoScreen.openUniversalLink(expiredDisclosureUniversalLinkFromCameraApp)
        finishWalletDialog.clickOkButton()
        assertTrue(demoScreen.visible(), "demo screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC6 Invalid universal link results in error screen")
    fun verifyInvalidUniversalLink(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Dashboard)
        val invalidIssuanceUniversalLink = "https://app.example.com/deeplink/disclosure_based_issuance?request_uri=https%3A%2F%2Fexample.com%2Fcd96997cf3772b54a9a0c9f2d261a401%2Fdisclosure%2Finsurance%2Frequest_uri%3Fsession_type%3Dsame_device&request_uri_method=post&client_id=fake.example.com"
        dashboardScreen.openUniversalLink(invalidIssuanceUniversalLink)
        assertAll(
            { assertTrue(invalidIssuanceULErrorScreen.headlineVisible(), "Headline is not visible") },
            { assertTrue(invalidIssuanceULErrorScreen.closeButtonVisible(), "Close button is not visible") },
        )
        invalidIssuanceULErrorScreen.errorDetails.seeDetails()
        assertAll(
            { assertTrue(invalidIssuanceULErrorScreen.errorDetails.appVersionLabelVisible(), "App version label is not visible") },
            { assertTrue(invalidIssuanceULErrorScreen.errorDetails.osVersionLabelVisible(), "OS version label is not visible") },
            { assertTrue(invalidIssuanceULErrorScreen.errorDetails.appConfigLabelVisible(), "App config label is not visible") },
            { assertTrue(invalidIssuanceULErrorScreen.errorDetails.appVersionVisible(), "App version is not visible") },
            { assertTrue(invalidIssuanceULErrorScreen.errorDetails.osVersionVisible(), "OS version is not visible") },
            { assertTrue(invalidIssuanceULErrorScreen.errorDetails.appConfigVisible(), "App config is not visible") }
        )
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC69 universal link is invoked while wallet is being personalized")
    fun verifyWhenAppIsBeingPersonalized(testInfo: TestInfo) {
        setUp(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.SecurityChoosePin)
        pinScreen.openUniversalLink(expiredDisclosureUniversalLinkFromCameraApp)
        assertTrue(finishWalletDialog.visible(), "Finish wallet dialog is not visible")
    }
}
