package feature.disclosure

import helper.CardMetadataHelper
import helper.GbaDataHelper
import helper.GbaDataHelper.Field.FIRST_NAME
import helper.GbaDataHelper.Field.NAME
import helper.LocalizationHelper
import helper.OrganizationAuthMetadataHelper
import helper.OrganizationAuthMetadataHelper.Organization.AMSTERDAM
import helper.OrganizationAuthMetadataHelper.Organization.MARKETPLACE
import helper.OrganizationAuthMetadataHelper.Organization.MONKEYBIKE
import helper.OrganizationAuthMetadataHelper.Organization.XYZ
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
import screen.web.rp.RelyingPartyAmsterdamWebPage
import screen.web.rp.RelyingPartyMarketplaceWebPage
import screen.web.rp.RelyingPartyMonkeyBikeWebPage
import screen.web.rp.RelyingPartyOverviewWebPage
import screen.web.rp.RelyingPartyXyzBankWebPage

class DisclosureTests : TestBase() {

    companion object {
        const val USE_CASE = "Demo use cases"
        const val JIRA_ID = "PVW-2128"
    }

    private lateinit var overviewWebPage: RelyingPartyOverviewWebPage
    private lateinit var xyzBankWebPage: RelyingPartyXyzBankWebPage
    private lateinit var amsterdamWebPage: RelyingPartyAmsterdamWebPage
    private lateinit var marketPlaceWebPage: RelyingPartyMarketplaceWebPage
    private lateinit var disclosureScreen: DisclosureApproveOrganizationScreen
    private lateinit var monkeyBikeWebPage: RelyingPartyMonkeyBikeWebPage
    private lateinit var pinScreen: PinScreen
    private lateinit var l10n: LocalizationHelper
    private lateinit var cardMetadata: CardMetadataHelper
    private lateinit var organizationAuthMetadata: OrganizationAuthMetadataHelper
    private lateinit var gbaData: GbaDataHelper


    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        overviewWebPage = RelyingPartyOverviewWebPage()
        disclosureScreen = DisclosureApproveOrganizationScreen()
        pinScreen = PinScreen()
        l10n = LocalizationHelper()
        cardMetadata = CardMetadataHelper()
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
        disclosureScreen.viewDisclosureDetails()
        assertTrue(disclosureScreen.organizationDescriptionOnDetailsVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.description", XYZ)))
        disclosureScreen.goBack();
        disclosureScreen.cancel()
        disclosureScreen.reportProblem()
        assertTrue(disclosureScreen.reportOptionUnknownOrganizationVisible(), "Reporting option not visible")
        disclosureScreen.goBack()
        disclosureScreen.proceed()
        disclosureScreen.share()
        pinScreen.enterPin(OnboardingNavigator.PIN)
        disclosureScreen.close()
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
        disclosureScreen.viewSharedData("1", cardMetadata.getPidDisplayName())
        assertTrue(disclosureScreen.bsnVisible("999991772"), "BSN not visible")
        disclosureScreen.goBack()
        disclosureScreen.goBack()
        disclosureScreen.cancel()
        disclosureScreen.reportProblem()
        assertTrue(disclosureScreen.reportOptionUnknownOrganizationVisible(), "Reporting option not visible")
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
        disclosureScreen.viewDisclosureDetails()
        assertTrue(disclosureScreen.organizationDescriptionOnDetailsVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.description", MARKETPLACE)))
        disclosureScreen.goBack();
        disclosureScreen.proceed()
        assertAll(
            { assertTrue(disclosureScreen.organizationInPresentationRequestHeaderVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", MARKETPLACE)), "Header is not visible") },
            { assertTrue(disclosureScreen.labelVisible(cardMetadata.getPidClaimLabel("family_name")), "Label is not visible") },
            { assertTrue(disclosureScreen.labelVisible(cardMetadata.getPidClaimLabel("given_name")), "Label is not visible") },
            { assertTrue(disclosureScreen.dataNotVisible(gbaData.getValueByField(NAME, "999991772")), "data is visible") },
            { assertTrue(disclosureScreen.dataNotVisible(gbaData.getValueByField(FIRST_NAME, "999991772")), "data is visible") },
            { assertTrue(disclosureScreen.sharingReasonVisible(organizationAuthMetadata.getAttributeValueForOrganization("purposeStatement", MARKETPLACE)), "reason is not visible") },
            { assertTrue(disclosureScreen.conditionsHeaderVisible(), "Description is not visible") },
            { assertTrue(disclosureScreen.conditionsButtonVisible(), "Try again button is not visible") }
        )
        disclosureScreen.viewSharedData("3", cardMetadata.getPidDisplayName())
        assertTrue(disclosureScreen.dataVisible(gbaData.getValueByField(NAME, "999991772")), "Name not visible")
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
        disclosureScreen.close()
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
        assertTrue(disclosureScreen.organizationNameForSharingFlowVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", MONKEYBIKE)))
        disclosureScreen.viewDisclosureDetails()
        assertTrue(disclosureScreen.organizationDescriptionOnDetailsVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.description", MONKEYBIKE)))
        disclosureScreen.goBack();
        disclosureScreen.proceed()
        assertTrue(disclosureScreen.attributesMissingMessageVisible(), "Attributes missing message not visible")
        disclosureScreen.stopRequestAfterMissingAttributeFailure()
        disclosureScreen.closeDisclosureAfterCompletedOrUncompleted()
        monkeyBikeWebPage.switchToWebViewContext()
        assertTrue(monkeyBikeWebPage.loginFailedMessageVisible(), "Login failed message not visible")
    }
}
