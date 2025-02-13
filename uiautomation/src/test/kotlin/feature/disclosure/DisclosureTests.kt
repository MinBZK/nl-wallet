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

    @BeforeEach
    fun setUp() {
        overviewWebPage = RelyingPartyOverviewWebPage()
        disclosureScreen = DisclosureApproveOrganizationScreen()
        pinScreen = PinScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("Opening a bank account")
    fun verifyDisclosureCreateAccountXyzBank() {
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        xyzBankWebPage = RelyingPartyXyzBankWebPage()
        overviewWebPage.clickXyzBankButton()
        val platform = overviewWebPage.platformName()
        xyzBankWebPage.openSameDeviceWalletFlow(platform)
        xyzBankWebPage.switchToAppContext()
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
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        amsterdamWebPage = RelyingPartyAmsterdamWebPage()
        overviewWebPage.clickAmsterdamButton()
        val platform = overviewWebPage.platformName()
        amsterdamWebPage.openSameDeviceWalletFlow(platform)
        amsterdamWebPage.switchToAppContext()
        disclosureScreen.login()
        pinScreen.enterPin(OnboardingNavigator.PIN)
        disclosureScreen.goToWebsite()
        assertTrue(amsterdamWebPage.loggedInMessageVisible(), "User not logged in correctly")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("Sign up, Online Marketplace")
    fun verifyDisclosureCreateAccountMarketplace() {
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        marketPlaceWebPage = RelyingPartyMarketplaceWebPage()
        overviewWebPage.clickMarketplaceButton()
        val platform = overviewWebPage.platformName()
        marketPlaceWebPage.openSameDeviceWalletFlow(platform)
        marketPlaceWebPage.switchToAppContext()
        disclosureScreen.proceed()
        disclosureScreen.share()
        pinScreen.enterPin(OnboardingNavigator.PIN)
        disclosureScreen.close()
        assertTrue(marketPlaceWebPage.welcomeMessageVisible(), "User not identified correctly")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("Sign up, MonkeyBike")
    fun verifyDisclosureCreateAccountMonkeyBike() {
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        monkeyBikeWebPage = RelyingPartyMonkeyBikeWebPage()
        overviewWebPage.clickMonkeyBikeButton()
        val platform = overviewWebPage.platformName()
        monkeyBikeWebPage.openSameDeviceWalletFlow(platform)
        monkeyBikeWebPage.switchToAppContext()
        disclosureScreen.proceed()
        assertTrue(disclosureScreen.attributesMissingMessageVisible(), "Attributes missing message not visible")
    }
}
