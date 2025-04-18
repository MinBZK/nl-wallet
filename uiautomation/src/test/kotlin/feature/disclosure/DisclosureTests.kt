package feature.disclosure

import helper.LocalizationHelper
import helper.LocalizationHelper.Translation.CREATE_ACCOUNT_SHARING_REASON
import helper.LocalizationHelper.Translation.FIRST_NAME_LABEL
import helper.LocalizationHelper.Translation.NAME_LABEL
import helper.LocalizationHelper.Translation.PID_CARD_TITLE
import helper.LocalizationHelper.Translation.AMSTERDAM_DISPLAY_NAME
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


    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        overviewWebPage = RelyingPartyOverviewWebPage()
        disclosureScreen = DisclosureApproveOrganizationScreen()
        pinScreen = PinScreen()
        l10n = LocalizationHelper()
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
        assertTrue(disclosureScreen.organizationNameForSharingFlowVisible("XYZ Bank"))
        disclosureScreen.viewDisclosureDetails()
        assertTrue(disclosureScreen.organizationDescriptionOnDetailsVisible("De toegankelijke bank voor betalen, sparen en beleggen."))
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
        assertTrue(disclosureScreen.organizationNameForLoginFlowVisible(l10n.translate(AMSTERDAM_DISPLAY_NAME)))
        disclosureScreen.viewLoginDisclosureDetails()
        disclosureScreen.viewOrganization(l10n.translate(AMSTERDAM_DISPLAY_NAME));
        val organizationDetailScreen = OrganizationDetailScreen()
        organizationDetailScreen.clickBackButton()
        disclosureScreen.viewSharedData("1",l10n.translate(PID_CARD_TITLE))
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
        assertTrue(disclosureScreen.organizationNameForSharingFlowVisible("Marktplek"))
        disclosureScreen.viewDisclosureDetails()
        assertTrue(disclosureScreen.organizationDescriptionOnDetailsVisible("Verkoop eenvoudig jouw tweedehands spullen online op Marktplek."))
        disclosureScreen.goBack();
        disclosureScreen.proceed()
        assertAll(
            { assertTrue(disclosureScreen.organizationInPresentationRequestHeaderVisible("Marktplek"), "Header is not visible") },
            { assertTrue(disclosureScreen.labelVisible(l10n.translate(NAME_LABEL)), "Label is not visible") },
            { assertTrue(disclosureScreen.labelVisible(l10n.translate(FIRST_NAME_LABEL)), "Label is not visible") },
            { assertTrue(disclosureScreen.dataNotVisible("Jansen"), "data is visible") },
            { assertTrue(disclosureScreen.dataNotVisible("Frouke"), "data is visible") },
            { assertTrue(disclosureScreen.sharingReasonVisible(l10n.translate(CREATE_ACCOUNT_SHARING_REASON)), "reason is not visible") },
            { assertTrue(disclosureScreen.conditionsHeaderVisible(), "Description is not visible") },
            { assertTrue(disclosureScreen.conditionsButtonVisible(), "Try again button is not visible") }
        )
        disclosureScreen.viewSharedData("3",l10n.translate(PID_CARD_TITLE))
        assertTrue(disclosureScreen.bsnVisible("Jansen"), "Name not visible")
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
        assertTrue(disclosureScreen.organizationNameForSharingFlowVisible("MonkeyBike"))
        disclosureScreen.viewDisclosureDetails()
        assertTrue(disclosureScreen.organizationDescriptionOnDetailsVisible("Jouw boodschappen binnen 10 minuten thuisbezorgd."))
        disclosureScreen.goBack();
        disclosureScreen.proceed()
        assertTrue(disclosureScreen.attributesMissingMessageVisible(), "Attributes missing message not visible")
        disclosureScreen.stopRequestAfterMissingAttributeFailure()
        disclosureScreen.closeDisclosureAfterCompletedOrUncompleted()
        monkeyBikeWebPage.switchToWebViewContext()
        assertTrue(monkeyBikeWebPage.loginFailedMessageVisible(), "Login failed message not visible")
    }
}
