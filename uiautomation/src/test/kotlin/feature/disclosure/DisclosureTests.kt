package feature.disclosure

import helper.GbaDataHelper
import helper.GbaDataHelper.Field.FIRST_NAME
import helper.GbaDataHelper.Field.NAME
import helper.IssuanceDataHelper
import helper.LocalizationHelper
import helper.OrganizationAuthMetadataHelper
import helper.OrganizationAuthMetadataHelper.Organization.AMSTERDAM
import helper.OrganizationAuthMetadataHelper.Organization.JOBFINDER
import helper.OrganizationAuthMetadataHelper.Organization.MARKETPLACE
import helper.OrganizationAuthMetadataHelper.Organization.XYZ
import helper.TasDataHelper
import helper.TestBase
import navigator.MenuNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.assertAll
import org.junitpioneer.jupiter.RetryingTest
import screen.dashboard.DashboardScreen
import screen.disclosure.DisclosureApproveOrganizationScreen
import screen.disclosure.SharingStoppedScreen
import screen.issuance.CardIssuanceScreen
import screen.issuance.DisclosureIssuanceScreen
import screen.menu.MenuScreen
import screen.organization.OrganizationDetailScreen
import screen.security.PinScreen
import screen.web.demo.DemoIndexWebPage
import screen.web.demo.issuer.IssuerWebPage
import screen.web.demo.rp.RelyingPartyAmsterdamWebPage
import screen.web.demo.rp.RelyingPartyJobFinderWebPage
import screen.web.demo.rp.RelyingPartyMarketplaceWebPage
import screen.web.demo.rp.RelyingPartyMonkeyBikeWebPage
import screen.web.demo.rp.RelyingPartyXyzBankWebPage

class DisclosureTests : TestBase() {

    private lateinit var indexWebPage: DemoIndexWebPage
    private lateinit var xyzBankWebPage: RelyingPartyXyzBankWebPage
    private lateinit var amsterdamWebPage: RelyingPartyAmsterdamWebPage
    private lateinit var marketPlaceWebPage: RelyingPartyMarketplaceWebPage
    private lateinit var disclosureScreen: DisclosureApproveOrganizationScreen
    private lateinit var monkeyBikeWebPage: RelyingPartyMonkeyBikeWebPage
    private lateinit var pinScreen: PinScreen
    private lateinit var l10n: LocalizationHelper
    private lateinit var tasData: TasDataHelper
    private lateinit var organizationAuthMetadata: OrganizationAuthMetadataHelper
    private lateinit var gbaData: GbaDataHelper
    private lateinit var organizationDetailScreen: OrganizationDetailScreen
    private lateinit var issuerWebPage: IssuerWebPage
    private lateinit var disclosureForIssuanceScreen: DisclosureIssuanceScreen
    private lateinit var cardIssuanceScreen: CardIssuanceScreen
    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var issuanceData : IssuanceDataHelper
    private lateinit var jobFinderWebPage : RelyingPartyJobFinderWebPage
    private lateinit var sharingStoppedScreen: SharingStoppedScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        indexWebPage = DemoIndexWebPage()
        disclosureScreen = DisclosureApproveOrganizationScreen()
        organizationDetailScreen = OrganizationDetailScreen()
        pinScreen = PinScreen()
        xyzBankWebPage = RelyingPartyXyzBankWebPage()
        amsterdamWebPage = RelyingPartyAmsterdamWebPage()
        marketPlaceWebPage = RelyingPartyMarketplaceWebPage()
        monkeyBikeWebPage = RelyingPartyMonkeyBikeWebPage()
        issuerWebPage = IssuerWebPage()
        disclosureForIssuanceScreen = DisclosureIssuanceScreen()
        dashboardScreen = DashboardScreen()
        jobFinderWebPage = RelyingPartyJobFinderWebPage()
        cardIssuanceScreen = CardIssuanceScreen()
        sharingStoppedScreen = SharingStoppedScreen()

        l10n = LocalizationHelper()
        tasData = TasDataHelper()
        organizationAuthMetadata = OrganizationAuthMetadataHelper()
        gbaData = GbaDataHelper()
        issuanceData = IssuanceDataHelper()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC15 Share data flow, Opening a bank account. MDOC")
    @Tags(Tag("a11yBatch1"))
    fun verifyDisclosureCreateAccountXyzBankMdoc(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        indexWebPage.switchToWebViewContext()
        indexWebPage.clickXyzBankMdocButton()
        xyzBankWebPage.openSameDeviceWalletFlow()
        xyzBankWebPage.switchToNativeContext()
        assertTrue(disclosureScreen.organizationNameForSharingFlowVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", XYZ)))

        disclosureScreen.viewDisclosureOrganizationDetails()
        assertTrue(disclosureScreen.organizationDescriptionOnDetailsVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.description", XYZ)))

        disclosureScreen.goBack();
        disclosureScreen.cancel()
        disclosureScreen.reportProblem()
        assertTrue(disclosureScreen.reportOptionSuspiciousVisible(), "Reporting option not visible")

        disclosureScreen.goBack()
        disclosureScreen.share()
        pinScreen.enterPin(DEFAULT_PIN)
        disclosureScreen.goToWebsite()
        assertTrue(xyzBankWebPage.sharedAttributeVisible(DEFAULT_BSN), "User not identified correctly")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC15 Share data flow, Opening a bank account. SD-JWT")
    fun verifyDisclosureCreateAccountXyzBankSdJwt(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        indexWebPage.switchToWebViewContext()
        indexWebPage.clickXyzBankSdJwtButton()
        xyzBankWebPage.openSameDeviceWalletFlow()
        xyzBankWebPage.switchToNativeContext()
        assertTrue(disclosureScreen.organizationNameForSharingFlowVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", XYZ)))
        disclosureScreen.viewDisclosureOrganizationDetails()
        assertTrue(disclosureScreen.organizationDescriptionOnDetailsVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.description", XYZ)))
        disclosureScreen.goBack();
        disclosureScreen.cancel()
        disclosureScreen.reportProblem()
        assertTrue(disclosureScreen.reportOptionSuspiciousVisible(), "Reporting option not visible")
        disclosureScreen.goBack()
        disclosureScreen.share()
        pinScreen.enterPin(DEFAULT_PIN)
        disclosureScreen.goToWebsite()
        assertTrue(xyzBankWebPage.sharedAttributeVisible(DEFAULT_BSN), "User not identified correctly")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC18 LTC25 RP Login flow, MDOC")
    @Tags(Tag("smoke"), Tag("a11yBatch1"))
    fun verifyDisclosureLoginMdoc(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        amsterdamWebPage.switchToWebViewContext()
        amsterdamWebPage = RelyingPartyAmsterdamWebPage()
        indexWebPage.clickAmsterdamMdocButton()
        amsterdamWebPage.openSameDeviceWalletFlow()
        amsterdamWebPage.switchToNativeContext()
        assertTrue(disclosureScreen.organizationNameForLoginFlowVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", AMSTERDAM)))

        disclosureScreen.viewLoginDisclosureDetails()
        disclosureScreen.viewOrganization(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", AMSTERDAM)
            .plus("\n" + organizationAuthMetadata.getAttributeValueForOrganization("organization.category", AMSTERDAM)))

        organizationDetailScreen.clickBackButton()
        disclosureScreen.viewSharedData("1", tasData.getPidDisplayName())
        assertTrue(disclosureScreen.bsnVisible(DEFAULT_BSN.toCharArray().joinToString(" ")), "BSN not visible")

        disclosureScreen.goBack()
        disclosureScreen.goBack()
        disclosureScreen.cancel()
        disclosureScreen.reportProblem()
        assertTrue(disclosureScreen.reportOptionSuspiciousVisible(), "Reporting option not visible")

        disclosureScreen.goBack()
        disclosureScreen.viewLoginDisclosureDetails()
        disclosureScreen.readTerms()
        assertTrue(disclosureScreen.termsVisible(), "Terms not visible")

        disclosureScreen.goBack()
        disclosureScreen.goBack()
        disclosureScreen.login()
        pinScreen.enterPin(DEFAULT_PIN)
        disclosureScreen.goToWebsite()
        assertTrue(amsterdamWebPage.loggedInMessageVisible(), "User not logged in correctly")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC18 LTC25 RP Login flow, SD-JWT")
    @Tags(Tag("smoke"))
    fun verifyDisclosureLoginSdJwt(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        amsterdamWebPage.switchToWebViewContext()
        amsterdamWebPage = RelyingPartyAmsterdamWebPage()
        indexWebPage.clickAmsterdamSdJwtButton()
        amsterdamWebPage.openSameDeviceWalletFlow()
        amsterdamWebPage.switchToNativeContext()
        assertTrue(disclosureScreen.organizationNameForLoginFlowVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", AMSTERDAM)))
        disclosureScreen.viewLoginDisclosureDetails()
        disclosureScreen.viewOrganization(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", AMSTERDAM).plus("\nGemeente"))
        organizationDetailScreen.clickBackButton()
        disclosureScreen.viewSharedData("1", tasData.getPidDisplayName())
        assertTrue(disclosureScreen.bsnVisible(DEFAULT_BSN.toCharArray().joinToString(" ")), "BSN not visible")
        disclosureScreen.goBack()
        disclosureScreen.goBack()
        disclosureScreen.cancel()
        disclosureScreen.reportProblem()
        assertTrue(disclosureScreen.reportOptionSuspiciousVisible(), "Reporting option not visible")
        disclosureScreen.goBack()
        disclosureScreen.viewLoginDisclosureDetails()
        disclosureScreen.readTerms()
        assertTrue(disclosureScreen.termsVisible(), "Terms not visible")
        disclosureScreen.goBack()
        disclosureScreen.goBack()
        disclosureScreen.login()
        pinScreen.enterPin(DEFAULT_PIN)
        disclosureScreen.goToWebsite()
        assertTrue(amsterdamWebPage.loggedInMessageVisible(), "User not logged in correctly")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC15 Share data flow")
    fun verifyDisclosureCreateAccountMarketplace(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        indexWebPage.switchToWebViewContext()
        indexWebPage.clickMarketplaceButton()
        marketPlaceWebPage.openSameDeviceWalletFlow()
        marketPlaceWebPage.switchToNativeContext()
        assertTrue(disclosureScreen.organizationNameForSharingFlowVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", MARKETPLACE)))
        disclosureScreen.viewDisclosureOrganizationDetails()
        assertTrue(disclosureScreen.organizationDescriptionOnDetailsVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.description", MARKETPLACE)))
        disclosureScreen.goBack();
        assertAll(
            { assertTrue(disclosureScreen.organizationInPresentationRequestHeaderVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", MARKETPLACE)), "Header is not visible") },
            { assertTrue(disclosureScreen.labelVisible(tasData.getPidClaimLabel("family_name")), "Label is not visible") },
            { assertTrue(disclosureScreen.labelVisible(tasData.getPidClaimLabel("given_name")), "Label is not visible") },
            { assertTrue(disclosureScreen.dataNotVisible(gbaData.getValueByField(NAME, DEFAULT_BSN)), "data is visible") },
            { assertTrue(disclosureScreen.dataNotVisible(gbaData.getValueByField(FIRST_NAME, DEFAULT_BSN)), "data is visible") },
            { assertTrue(disclosureScreen.sharingReasonVisible(organizationAuthMetadata.getAttributeValueForOrganization("purposeStatement", MARKETPLACE)), "reason is not visible") },
            { assertTrue(disclosureScreen.conditionsHeaderVisible(), "Description is not visible") },
            { assertTrue(disclosureScreen.conditionsButtonVisible(), "Try again button is not visible") }
        )
        disclosureScreen.viewSharedData("4", tasData.getPidDisplayName())
        assertTrue(disclosureScreen.dataVisible(gbaData.getValueByField(NAME, DEFAULT_BSN)), "Name not visible")
        disclosureScreen.goBack()
        disclosureScreen.readTerms()
        assertTrue(disclosureScreen.termsVisible(), "Terms not visible")
        disclosureScreen.goBack()
        disclosureScreen.cancel()
        disclosureScreen.reportProblem()
        assertTrue(disclosureScreen.reportOptionSuspiciousVisible(), "Reporting option not visible")
        disclosureScreen.goBack()
        disclosureScreen.share()
        pinScreen.enterPin(DEFAULT_PIN)
        disclosureScreen.goToWebsite()
        assertTrue(marketPlaceWebPage.welcomeMessageVisible(), "User not identified correctly")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC10 Wallet does not contain requested attributes")
    @Tags(Tag("a11yBatch1"))
    fun verifyDisclosureCreateAccountMonkeyBike(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        indexWebPage.switchToWebViewContext()
        indexWebPage.clickMonkeyBikeButton()
        monkeyBikeWebPage.openSameDeviceWalletFlow()
        monkeyBikeWebPage.switchToNativeContext()
        assertTrue(disclosureScreen.attributesMissingMessageVisible(), "Attributes missing message not visible")

        disclosureScreen.stopRequestAfterMissingAttributeFailure()
        assertTrue(dashboardScreen.visible(), "App dashboard not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC65 User selects another card to be disclosed MDOC")
    fun verifyUserSelectCardToDisclose(testInfo: TestInfo) {
        setUp(testInfo)

        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        indexWebPage.switchToWebViewContext()
        indexWebPage.clickHollandUniversityMdocButton()
        issuerWebPage.openSameDeviceWalletFlow()

        disclosureForIssuanceScreen.switchToNativeContext()
        disclosureForIssuanceScreen.share()
        pinScreen.enterPin(DEFAULT_PIN)
        cardIssuanceScreen.clickAdd2CardsButton()
        pinScreen.enterPin(DEFAULT_PIN)
        cardIssuanceScreen.clickToDashboardButton()

        dashboardScreen.clickMenuButton()
        MenuScreen().clickBrowserTestButton()
        indexWebPage.switchToWebViewContext()
        indexWebPage.clickJobFinderButton()
        jobFinderWebPage.openSameDeviceWalletFlow()
        jobFinderWebPage.switchToNativeContext()
        disclosureScreen.viewDisclosureOrganizationDetails()
        assertTrue(disclosureScreen.organizationDescriptionOnDetailsVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.description", JOBFINDER)))
        disclosureScreen.goBack();
        disclosureScreen.clickSwapCardButton()
        disclosureScreen.swapCardTo(issuanceData.getAttributeValues("university", DEFAULT_BSN, "education").last())
        disclosureScreen.share()
        pinScreen.enterPin(DEFAULT_PIN)
        disclosureScreen.goToWebsite()
        assertTrue(jobFinderWebPage.sharedAttributeVisible(issuanceData.getAttributeValues("university", DEFAULT_BSN, "education").last()), "Attribute of selected card not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC15 Share data flow, Opening a bank account. SD-JWT, extended VCT")
    fun verifyDisclosureCreateAccountXyzBankSdJwtExtendingVct(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        indexWebPage.switchToWebViewContext()
        indexWebPage.clickXyzBankSdJwtEuPidButton()
        xyzBankWebPage.openSameDeviceWalletFlow()
        xyzBankWebPage.switchToNativeContext()
        assertTrue(disclosureScreen.organizationNameForSharingFlowVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", XYZ)))
        disclosureScreen.share()
        pinScreen.enterPin(DEFAULT_PIN)
        disclosureScreen.goToWebsite()
        assertTrue(xyzBankWebPage.sharedAttributeVisible(gbaData.getValueByField(FIRST_NAME, DEFAULT_BSN)), "User not identified correctly")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC17 Decline consent to share data")
    fun verifyDeclineConsent(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        indexWebPage.switchToWebViewContext()
        indexWebPage.clickAmsterdamSdJwtButton()
        amsterdamWebPage.openSameDeviceWalletFlow()
        amsterdamWebPage.switchToNativeContext()
        disclosureScreen.cancel()
        disclosureScreen.stop()
        disclosureScreen.bottomSheetConfirmStop()
        sharingStoppedScreen.close()
        assertTrue(dashboardScreen.visible(), "Dashboard not visible")
    }
}
