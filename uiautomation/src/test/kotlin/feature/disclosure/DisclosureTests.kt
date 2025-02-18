package feature.disclosure

import helper.TestBase
import navigator.MenuNavigator
import navigator.OnboardingNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.Test
import org.junitpioneer.jupiter.RetryingTest
import screen.disclosure.DisclosureApproveOrganizationScreen
import screen.menu.MenuScreen
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


    fun setUp() {
        overviewWebPage = RelyingPartyOverviewWebPage()
        disclosureScreen = DisclosureApproveOrganizationScreen()
        pinScreen = PinScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("Opening a bank account")
    fun verifyDisclosureCreateAccountXyzBank() {
        setUp()
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
        disclosureScreen.proceed()
        disclosureScreen.share()
        pinScreen.enterPin(OnboardingNavigator.PIN)
        disclosureScreen.close()
        assertTrue(xyzBankWebPage.identificationSucceededMessageVisible(), "User not identified correctly")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("Log in to MijnAmsterdam")
    @Tags(Tag("smoke"))
    fun verifyDisclosureLogin() {
        setUp()
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        amsterdamWebPage = RelyingPartyAmsterdamWebPage()
        overviewWebPage.clickAmsterdamButton()
        val platform = overviewWebPage.platformName()
        amsterdamWebPage.openSameDeviceWalletFlow(platform)
        amsterdamWebPage.switchToAppContext()
        assertTrue(disclosureScreen.organizationNameForLoginFlowVisible("Gemeente Amsterdam"))
        disclosureScreen.viewLoginDisclosureDetails()
        disclosureScreen.goBack();
        disclosureScreen.login()
        pinScreen.enterPin(OnboardingNavigator.PIN)
        disclosureScreen.goToWebsite()
        assertTrue(amsterdamWebPage.loggedInMessageVisible(), "User not logged in correctly")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("Sign up, Online Marketplace")
    fun verifyDisclosureCreateAccountMarketplace() {
        setUp()
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
        disclosureScreen.share()
        pinScreen.enterPin(OnboardingNavigator.PIN)
        disclosureScreen.close()
        assertTrue(marketPlaceWebPage.welcomeMessageVisible(), "User not identified correctly")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("Sign up, MonkeyBike")
    fun verifyDisclosureCreateAccountMonkeyBike() {
        setUp()
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
    }
}
