package feature.issuance

import helper.IssuanceDataHelper
import helper.OrganizationAuthMetadataHelper
import helper.OrganizationAuthMetadataHelper.Organization.UNIVERSITY
import helper.TasDataHelper
import helper.TwoDeviceTestBase
import navigator.MenuNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import screen.dashboard.DashboardScreen
import screen.disclosure.QRScanner
import screen.disclosure.UrlCheckScreen
import screen.issuance.CardIssuanceScreen
import screen.issuance.DisclosureIssuanceScreen
import screen.security.PinScreen
import screen.web.demo.DemoIndexWebPage
import screen.web.demo.issuer.IssuerWebPage
import util.EnvironmentUtil
import util.MobileActions

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("Cross device issuance")
class CrossDeviceDisclosureBasedIssuanceTests : TwoDeviceTestBase() {

    companion object {
        private val DEMO_INDEX_URL = EnvironmentUtil.getVar("DEMO_INDEX_URL")
    }

    private lateinit var sourceDashboard: DashboardScreen
    private lateinit var sourceQrScanner: QRScanner
    private lateinit var sourceDisclosureScreen: DisclosureIssuanceScreen
    private lateinit var sourcePinScreen: PinScreen
    private lateinit var sourceUrlCheckScreen: UrlCheckScreen
    private lateinit var sourceCardIssuanceScreen: CardIssuanceScreen

    private lateinit var targetIndexWebPage: DemoIndexWebPage
    private lateinit var targetIssuerWebPage: IssuerWebPage

    private lateinit var issuanceData: IssuanceDataHelper
    private lateinit var tasData: TasDataHelper
    private lateinit var organizationAuthMetadata: OrganizationAuthMetadataHelper

    fun setUp(testInfo: TestInfo) {
        startDrivers(testInfo)

        issuanceData = IssuanceDataHelper()
        tasData = TasDataHelper()
        organizationAuthMetadata = OrganizationAuthMetadataHelper()

        useSourceDevice {
            sourceDashboard = DashboardScreen()
            sourceQrScanner = QRScanner()
            sourceDisclosureScreen = DisclosureIssuanceScreen()
            sourcePinScreen = PinScreen()
            sourceUrlCheckScreen = UrlCheckScreen()
            sourceCardIssuanceScreen = CardIssuanceScreen()
            MenuNavigator().toScreen(MenuNavigatorScreen.Dashboard)
        }

        useTargetDevice {
            targetIndexWebPage = DemoIndexWebPage()
            targetIssuerWebPage = IssuerWebPage()
            targetIndexWebPage.openUrlInBrowser(DEMO_INDEX_URL)
            targetIndexWebPage.switchToWebViewContext()
        }
    }

    @Test
    @DisplayName("LTC7 Cross-device disclosure based issuance, university, SD-JWT")
    @Tags(Tag("twoDevice"))
    fun verifyCrossDeviceDisclosureBasedIssuanceUniversitySdJwt(testInfo: TestInfo) {
        setUp(testInfo)

        useTargetDevice {
            targetIndexWebPage.clickHollandUniversitySdJwtButton()
            Thread.sleep(MobileActions.SET_FRAME_SYNC_MAX_WAIT_MILLIS)
            targetIssuerWebPage.openCrossDeviceWalletFlow()
        }

        useSourceDevice {
            sourceDashboard.openQRScanner()
            Thread.sleep(MobileActions.SET_FRAME_SYNC_MAX_WAIT_MILLIS)
            assertTrue(sourceQrScanner.visible())

            assertTrue(sourceDisclosureScreen.organizationNameVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", UNIVERSITY)))

            sourceDisclosureScreen.viewDetails()
            assertTrue(sourceDisclosureScreen.requestedAttributeVisible(tasData.getPidClaimLabel("bsn")))

            sourceDisclosureScreen.goBack()
            sourceDisclosureScreen.share()
            sourcePinScreen.enterPin(DEFAULT_PIN)

            sourceCardIssuanceScreen.viewDetailsOfCard(issuanceData.getAttributeValues("university", DEFAULT_BSN, "education").last())
            sourceCardIssuanceScreen.clickBackButton()
            sourceCardIssuanceScreen.clickAdd2CardsButton()
            sourcePinScreen.enterPin(DEFAULT_PIN)

            sourceCardIssuanceScreen.clickToDashboardButton()
            sourceDashboard.scrollToEndOfScreen()
            assertTrue(sourceDashboard.cardVisible(tasData.getDiplomaDisplayName()), "Diploma card not visible on dashboard")
        }
    }
}
