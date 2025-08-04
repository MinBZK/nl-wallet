package feature.disclosure

import helper.GbaDataHelper
import helper.GbaDataHelper.Field.FIRST_NAME
import helper.GbaDataHelper.Field.NAME
import helper.LocalizationHelper
import helper.OrganizationAuthMetadataHelper
import helper.OrganizationAuthMetadataHelper.Organization.MARKETPLACE
import helper.OrganizationAuthMetadataHelper.Organization.AMSTERDAM
import helper.OrganizationAuthMetadataHelper.Organization.XYZ
import helper.TasDataHelper
import helper.TestBase
import navigator.MenuNavigator
import navigator.OnboardingNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertAll
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.TestInfo
import org.junitpioneer.jupiter.RetryingTest
import screen.disclosure.DisclosureApproveOrganizationScreen
import screen.menu.MenuScreen
import screen.organization.OrganizationDetailScreen
import screen.security.PinScreen
import screen.web.demo.DemoIndexWebPage
import screen.web.demo.rp.RelyingPartyAmsterdamWebPage
import screen.web.demo.rp.RelyingPartyMarketplaceWebPage
import screen.web.demo.rp.RelyingPartyMonkeyBikeWebPage
import screen.web.demo.rp.RelyingPartyXyzBankWebPage

class DisclosureTests : TestBase() {

    companion object {
        const val USE_CASE = "Demo use cases"
        const val JIRA_ID = "PVW-2128"
    }

    private lateinit var overviewWebPage: DemoIndexWebPage
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

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        overviewWebPage = DemoIndexWebPage()
        disclosureScreen = DisclosureApproveOrganizationScreen()
        pinScreen = PinScreen()
        l10n = LocalizationHelper()
        tasData = TasDataHelper()
        organizationAuthMetadata = OrganizationAuthMetadataHelper()
        gbaData = GbaDataHelper()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("Opening a bank account")
    fun verifyDisclosureCreateAccountXyzBank(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        xyzBankWebPage = RelyingPartyXyzBankWebPage()
        overviewWebPage.clickXyzBankButton()
        val platform = overviewWebPage.platformName()
        xyzBankWebPage.openSameDeviceWalletFlow(platform)
        xyzBankWebPage.switchToAppContext()
        assertTrue(disclosureScreen.organizationNameForSharingFlowVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", XYZ)))
        disclosureScreen.viewDisclosureOrganizationDetails()
        assertTrue(disclosureScreen.organizationDescriptionOnDetailsVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.description", XYZ)))
        disclosureScreen.goBack();
        disclosureScreen.cancel()
        disclosureScreen.reportProblem()
        assertTrue(disclosureScreen.reportOptionUntrustedVisible(), "Reporting option not visible")
        disclosureScreen.goBack()
        disclosureScreen.proceed()
        disclosureScreen.share()
        pinScreen.enterPin(OnboardingNavigator.PIN)
        disclosureScreen.goToWebsite()
        assertTrue(xyzBankWebPage.identificationSucceededMessageVisible(), "User not identified correctly")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("Log in to MijnAmsterdam")
    @Tags(Tag("smoke"))
    fun verifyDisclosureLogin(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        amsterdamWebPage = RelyingPartyAmsterdamWebPage()
        overviewWebPage.clickAmsterdamButton()
        val platform = overviewWebPage.platformName()
        amsterdamWebPage.openSameDeviceWalletFlow(platform)
        amsterdamWebPage.switchToAppContext()
        assertTrue(disclosureScreen.organizationNameForLoginFlowVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", AMSTERDAM)))
        disclosureScreen.viewLoginDisclosureDetails()
        disclosureScreen.viewOrganization(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", AMSTERDAM))
        val organizationDetailScreen = OrganizationDetailScreen()
        organizationDetailScreen.clickBackButton()
        disclosureScreen.viewSharedData("1", tasData.getPidDisplayName())
        assertTrue(disclosureScreen.bsnVisible(DEFAULT_BSN), "BSN not visible")
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
        pinScreen.enterPin(OnboardingNavigator.PIN)
        disclosureScreen.goToWebsite()
        assertTrue(amsterdamWebPage.loggedInMessageVisible(), "User not logged in correctly")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("Sign up, Online Marketplace")
    fun verifyDisclosureCreateAccountMarketplace(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        marketPlaceWebPage = RelyingPartyMarketplaceWebPage()
        overviewWebPage.clickMarketplaceButton()
        val platform = overviewWebPage.platformName()
        marketPlaceWebPage.openSameDeviceWalletFlow(platform)
        marketPlaceWebPage.switchToAppContext()
        assertTrue(disclosureScreen.organizationNameForSharingFlowVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", MARKETPLACE)))
        disclosureScreen.viewDisclosureOrganizationDetails()
        assertTrue(disclosureScreen.organizationDescriptionOnDetailsVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.description", MARKETPLACE)))
        disclosureScreen.goBack();
        disclosureScreen.proceed()
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
        assertTrue(disclosureScreen.reportOptionUntrustedVisible(), "Reporting option not visible")
        disclosureScreen.goBack()
        disclosureScreen.share()
        pinScreen.enterPin(OnboardingNavigator.PIN)
        disclosureScreen.goToWebsite()
        assertTrue(marketPlaceWebPage.welcomeMessageVisible(), "User not identified correctly")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("Sign up, MonkeyBike")
    fun verifyDisclosureCreateAccountMonkeyBike(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        monkeyBikeWebPage = RelyingPartyMonkeyBikeWebPage()
        overviewWebPage.clickMonkeyBikeButton()
        val platform = overviewWebPage.platformName()
        monkeyBikeWebPage.openSameDeviceWalletFlow(platform)
        monkeyBikeWebPage.switchToAppContext()
        assertTrue(disclosureScreen.attributesMissingMessageVisible(), "Attributes missing message not visible")
        disclosureScreen.stopRequestAfterMissingAttributeFailure()
        disclosureScreen.closeDisclosureAfterCompletedOrUncompleted()
        disclosureScreen.goToWebsite()
        monkeyBikeWebPage.switchToWebViewContext()
        assertTrue(monkeyBikeWebPage.loginFailedMessageVisible(), "Login failed message not visible")
    }
}
