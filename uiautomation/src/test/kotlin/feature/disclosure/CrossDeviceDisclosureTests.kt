package feature.disclosure

import helper.GbaDataHelper
import helper.GbaDataHelper.Field.FIRST_NAME
import helper.OrganizationAuthMetadataHelper
import helper.OrganizationAuthMetadataHelper.Organization.AMSTERDAM
import helper.OrganizationAuthMetadataHelper.Organization.XYZ
import helper.TasDataHelper
import helper.TwoDeviceTestBase
import navigator.MenuNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.dashboard.DashboardScreen
import screen.disclosure.DisclosureApproveOrganizationScreen
import screen.disclosure.QRScanner
import screen.disclosure.UrlCheckScreen
import screen.organization.OrganizationDetailScreen
import screen.security.PinScreen
import screen.web.demo.DemoIndexWebPage
import screen.web.demo.rp.RelyingPartyAmsterdamWebPage
import screen.web.demo.rp.RelyingPartyXyzBankWebPage
import util.EnvironmentUtil
import util.MobileActions

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("Cross device disclosure")
class CrossDeviceDisclosureTests : TwoDeviceTestBase() {

    companion object {
        private val DEMO_INDEX_URL = EnvironmentUtil.getVar("DEMO_INDEX_URL")
    }

    private lateinit var sourceDashboard: DashboardScreen
    private lateinit var sourceQrScanner: QRScanner
    private lateinit var sourceDisclosureScreen: DisclosureApproveOrganizationScreen
    private lateinit var sourcePinScreen: PinScreen
    private lateinit var sourceUrlCheckScreen: UrlCheckScreen
    private lateinit var sourceOrganizationDetailScreen: OrganizationDetailScreen

    private lateinit var targetIndexWebPage: DemoIndexWebPage
    private lateinit var targetXyzBankWebPage: RelyingPartyXyzBankWebPage
    private lateinit var targetAmsterdamWebPage: RelyingPartyAmsterdamWebPage

    private lateinit var gbaData: GbaDataHelper
    private lateinit var tasData: TasDataHelper
    private lateinit var organizationAuthMetadata: OrganizationAuthMetadataHelper

    fun setUp(testInfo: TestInfo) {
        startDrivers(testInfo)

        gbaData = GbaDataHelper()
        tasData = TasDataHelper()
        organizationAuthMetadata = OrganizationAuthMetadataHelper()

        useSourceDevice {
            sourceDashboard = DashboardScreen()
            sourceQrScanner = QRScanner()
            sourceDisclosureScreen = DisclosureApproveOrganizationScreen()
            sourcePinScreen = PinScreen()
            sourceUrlCheckScreen = UrlCheckScreen()
            sourceOrganizationDetailScreen = OrganizationDetailScreen()
            MenuNavigator().toScreen(MenuNavigatorScreen.Dashboard)
        }

        useTargetDevice {
            targetIndexWebPage = DemoIndexWebPage()
            targetXyzBankWebPage = RelyingPartyXyzBankWebPage()
            targetAmsterdamWebPage = RelyingPartyAmsterdamWebPage()
            targetIndexWebPage.openUrlInBrowser(DEMO_INDEX_URL)
            targetIndexWebPage.switchToWebViewContext()
        }
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC16 Cross-device data sharing")
    @Tags(Tag("a11yBatch3"), Tag("twoDevice"))
    fun verifyCrossDeviceDisclosureCreateAccountXyzBankMdoc(testInfo: TestInfo) {
        setUp(testInfo)

        useTargetDevice {
            targetIndexWebPage.clickXyzBankMdocButton()
            targetXyzBankWebPage.openCrossDeviceWalletFlow()
        }

        useSourceDevice {
            sourceDashboard.openQRScanner()
            Thread.sleep(MobileActions.SET_FRAME_SYNC_MAX_WAIT_MILLIS)
            assertTrue(sourceQrScanner.visible())

            assertTrue(sourceUrlCheckScreen.titleVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.webUrl", XYZ)))
            sourceUrlCheckScreen.clickContinueButton()
            assertTrue(sourceDisclosureScreen.organizationNameForSharingFlowVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", XYZ)))

            sourceDisclosureScreen.viewDisclosureOrganizationDetails()
            assertTrue(sourceDisclosureScreen.organizationDescriptionOnDetailsVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.description", XYZ)))

            sourceDisclosureScreen.goBack()
            sourceDisclosureScreen.cancel()
            sourceDisclosureScreen.reportProblem()
            assertTrue(sourceDisclosureScreen.reportOptionSuspiciousVisible(), "Reporting option not visible")

            sourceDisclosureScreen.goBack()
            sourceDisclosureScreen.share()
            sourcePinScreen.enterPin(DEFAULT_PIN)
            sourceDisclosureScreen.goToDashboard()
            assertTrue(sourceDashboard.visible(), "Dashboard not visible")
        }

        useTargetDevice {
            targetXyzBankWebPage.clickCloseButton()
            assertTrue(targetXyzBankWebPage.sharedAttributeVisible(gbaData.getValueByField(FIRST_NAME, DEFAULT_BSN)), "Shared attribute not visible on target device")
        }
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC19 Cross-device login")
    @Tags(Tag("twoDevice"))
    fun verifyDisclosureLoginMdoc(testInfo: TestInfo) {
        setUp(testInfo)

        useTargetDevice {
            targetIndexWebPage.clickAmsterdamMdocButton()
            targetAmsterdamWebPage.openCrossDeviceWalletFlow()
        }

        useSourceDevice {
            sourceDashboard.openQRScanner()
            Thread.sleep(MobileActions.SET_FRAME_SYNC_MAX_WAIT_MILLIS)
            assertTrue(sourceQrScanner.visible())

            assertTrue(sourceUrlCheckScreen.titleVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.webUrl", AMSTERDAM)))
            sourceUrlCheckScreen.clickContinueButton()
            assertTrue(sourceDisclosureScreen.organizationNameForLoginFlowVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", AMSTERDAM)))

            sourceDisclosureScreen.viewLoginDisclosureDetails()
            sourceDisclosureScreen.viewOrganization(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", AMSTERDAM)
                .plus("\n" + organizationAuthMetadata.getAttributeValueForOrganization("organization.category", AMSTERDAM)))

            sourceOrganizationDetailScreen.clickBackButton()
            sourceDisclosureScreen.viewSharedData("1", tasData.getPidDisplayName())
            assertTrue(sourceDisclosureScreen.bsnVisible(DEFAULT_BSN.toCharArray().joinToString(" ")), "BSN not visible")

            sourceDisclosureScreen.goBack()
            sourceDisclosureScreen.goBack()
            sourceDisclosureScreen.cancel()
            sourceDisclosureScreen.reportProblem()
            assertTrue(sourceDisclosureScreen.reportOptionSuspiciousVisible(), "Reporting option not visible")

            sourceDisclosureScreen.goBack()
            sourceDisclosureScreen.viewLoginDisclosureDetails()
            sourceDisclosureScreen.readTerms()
            assertTrue(sourceDisclosureScreen.termsVisible(), "Terms not visible")

            sourceDisclosureScreen.goBack()
            sourceDisclosureScreen.goBack()
            sourceDisclosureScreen.login()
            sourcePinScreen.enterPin(DEFAULT_PIN)
            sourceDisclosureScreen.goToDashboard()
            assertTrue(sourceDashboard.visible(), "Dashboard not visible")
        }

        useTargetDevice {
            targetXyzBankWebPage.clickCloseButton()
            assertTrue(targetAmsterdamWebPage.loggedInMessageVisible(), "Logged in message not visible on target device")
        }
    }
}
