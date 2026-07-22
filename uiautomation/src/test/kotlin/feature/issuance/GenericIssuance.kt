package feature.issuance

import helper.TasDataHelper
import helper.TestBase
import navigator.MenuNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.dashboard.DashboardScreen
import screen.issuance.CardIssuanceScreen
import screen.menu.MenuScreen
import screen.security.PinScreen
import screen.web.demo.DemoIndexWebPage
import screen.web.demo.issuer.IssuerConsentWebPage
import screen.web.demo.issuer.IssuerWebPage

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("Use Case 4.1 Obtain one or more cards from a (Q)EAA Issuer")
class GenericIssuance : TestBase() {

    private lateinit var indexWebPage: DemoIndexWebPage
    private lateinit var issuerWebPage: IssuerWebPage
    private lateinit var issuerConsentWebPage: IssuerConsentWebPage
    private lateinit var cardIssuanceScreen: CardIssuanceScreen
    private lateinit var pinScreen: PinScreen
    private lateinit var tasData: TasDataHelper
    private lateinit var dashboardScreen: DashboardScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        indexWebPage = DemoIndexWebPage()
        issuerWebPage = IssuerWebPage()
        issuerConsentWebPage = IssuerConsentWebPage()
        cardIssuanceScreen = CardIssuanceScreen()
        pinScreen = PinScreen()
        tasData = TasDataHelper()
        dashboardScreen = DashboardScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC83 Authorization code flow issuance")
    fun verifyAuthorizationCodeFlow(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        indexWebPage.switchToWebViewContext()
        indexWebPage.clickLoyaltyButton()
        issuerWebPage.openSameDeviceWalletFlow()
        issuerWebPage.acceptOpenWalletDialog()

        cardIssuanceScreen.switchToNativeContext()
        cardIssuanceScreen.clickAddCardButton()
        pinScreen.enterPin(DEFAULT_PIN)

        cardIssuanceScreen.clickToDashboardButton()
        dashboardScreen.scrollToEndOfScreen()
        assertTrue(dashboardScreen.cardVisible(tasData.getLoyaltyDisplayName()), "Loyalty card not visible on dashboard")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC84 Authorization code flow issuance with consent")
    fun verifyAuthorizationCodeFlowWithConsent(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        indexWebPage.switchToWebViewContext()
        indexWebPage.clickInsuranceButton()
        issuerWebPage.openSameDeviceWalletFlow()
        issuerWebPage.acceptOpenWalletDialog()

        issuerConsentWebPage.switchToWebViewContext()
        assertTrue(issuerConsentWebPage.visible(), "Issuer consent page not visible")
        issuerConsentWebPage.clickAddToWalletButton()

        cardIssuanceScreen.switchToNativeContext()
        cardIssuanceScreen.clickAddCardButton()
        pinScreen.enterPin(DEFAULT_PIN)

        cardIssuanceScreen.clickToDashboardButton()
        dashboardScreen.scrollToEndOfScreen()
        assertTrue(dashboardScreen.cardVisible(tasData.getInsuranceDisplayName()), "Insurance card not visible on dashboard")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC82 Pre-authorized code flow issuance")
    fun verifyPreauthorizedCodeFlow(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()

        indexWebPage.switchToWebViewContext()
        indexWebPage.clickMuseumMaandkaartButton()
        issuerWebPage.openSameDeviceWalletFlow()
        issuerWebPage.acceptOpenWalletDialog()

        cardIssuanceScreen.switchToNativeContext()
        cardIssuanceScreen.clickAddCardButton()
        pinScreen.enterPin(DEFAULT_PIN)

        cardIssuanceScreen.clickToDashboardButton()
        dashboardScreen.scrollToEndOfScreen()
        assertTrue(
            dashboardScreen.cardVisible(tasData.getMuseumMaandkaartDisplayName()),
            "Museum maandkaart card not visible on dashboard"
        )
    }
}
