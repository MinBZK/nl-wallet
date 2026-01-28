package feature.issuance

import helper.OrganizationAuthMetadataHelper
import helper.RevocationHelper
import helper.TasDataHelper
import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.card.CardDataScreen
import screen.card.CardDetailScreen
import screen.dashboard.DashboardScreen
import screen.error.AttributesMissingErrorScreen
import screen.issuance.CardIssuanceScreen
import screen.issuance.DisclosureIssuanceScreen
import screen.issuance.PersonalizeInformScreen
import screen.menu.MenuScreen
import screen.security.PinScreen
import screen.web.demo.DemoIndexWebPage
import screen.web.demo.issuer.IssuerWebPage

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("Show revocations")
class RevokeCardTests : TestBase() {

    private lateinit var indexWebPage: DemoIndexWebPage
    private lateinit var issuerWebPage: IssuerWebPage
    private lateinit var disclosureForIssuanceScreen: DisclosureIssuanceScreen
    private lateinit var cardIssuanceScreen: CardIssuanceScreen
    private lateinit var pinScreen: PinScreen
    private lateinit var organizationAuthMetadata: OrganizationAuthMetadataHelper
    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var personalizeInformScreen: PersonalizeInformScreen
    private lateinit var attributesMissingErrorScreen: AttributesMissingErrorScreen
    private lateinit var revocationHelper: RevocationHelper
    private lateinit var tasData: TasDataHelper
    private lateinit var cardDetailScreen: CardDetailScreen
    private lateinit var cardDataScreen: CardDataScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)

        indexWebPage = DemoIndexWebPage()
        issuerWebPage = IssuerWebPage()
        pinScreen = PinScreen()
        organizationAuthMetadata = OrganizationAuthMetadataHelper()
        disclosureForIssuanceScreen = DisclosureIssuanceScreen()
        cardIssuanceScreen = CardIssuanceScreen()
        dashboardScreen = DashboardScreen()
        personalizeInformScreen = PersonalizeInformScreen()
        attributesMissingErrorScreen = AttributesMissingErrorScreen()
        revocationHelper = RevocationHelper()
        cardDetailScreen = CardDetailScreen()
        cardDataScreen = CardDataScreen()
        tasData = TasDataHelper()

        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.Dashboard)
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC67 Revoke PID card")
    fun verifyPidRevocation(testInfo: TestInfo) {
        setUp(testInfo)
        dashboardScreen.closeApp()
        revocationHelper.revokeAllNonRevokedPids()
        dashboardScreen.openApp()
        pinScreen.enterPin(DEFAULT_PIN)
        assertTrue(dashboardScreen.cardRevocationVisible(tasData.getPidDisplayName()))

        dashboardScreen.clickCard(tasData.getPidDisplayName())
        cardDetailScreen.clickCardDataButton()
        assertTrue(cardDataScreen.revocationMessageVisible())

        cardDataScreen.clickBottomBackButton()
        cardDetailScreen.clickBottomBackButton()
        dashboardScreen.clickMenuButton()
        MenuScreen().clickBrowserTestButton()
        indexWebPage.switchToWebViewContext()
        indexWebPage.clickInsuranceButton()
        issuerWebPage.openSameDeviceWalletFlow()
        disclosureForIssuanceScreen.switchToNativeContext()
        assertTrue(attributesMissingErrorScreen.attributesMissingMessageVisible(), "Error screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC68 Revoke EEA Card")
    fun verifyEeaCardRevocation(testInfo: TestInfo) {
        setUp(testInfo)
        dashboardScreen.clickMenuButton()
        MenuScreen().clickBrowserTestButton()
        indexWebPage.switchToWebViewContext()
        indexWebPage.clickHollandUniversitySdJwtButton()
        issuerWebPage.openSameDeviceWalletFlow()
        disclosureForIssuanceScreen.switchToNativeContext()
        disclosureForIssuanceScreen.share()
        pinScreen.enterPin(DEFAULT_PIN)
        cardIssuanceScreen.clickAdd2CardsButton()
        pinScreen.enterPin(DEFAULT_PIN)
        cardIssuanceScreen.clickToDashboardButton()

        dashboardScreen.closeApp()
        revocationHelper.revokeAllNonRevokedEeaCards()
        dashboardScreen.openApp()
        pinScreen.enterPin(DEFAULT_PIN)
        assertTrue(dashboardScreen.cardRevocationVisible(tasData.getDiplomaDisplayName()))

        dashboardScreen.clickCard(tasData.getDiplomaDisplayName())
        cardDetailScreen.clickCardDataButton()
        assertTrue(cardDataScreen.revocationMessageVisible())

        cardDataScreen.clickBottomBackButton()
        cardDetailScreen.clickBottomBackButton()
        dashboardScreen.clickMenuButton()
        MenuScreen().clickBrowserTestButton()
        indexWebPage.switchToWebViewContext()
        indexWebPage.clickJobFinderButton()
        issuerWebPage.openSameDeviceWalletFlow()
        disclosureForIssuanceScreen.switchToNativeContext()
        assertTrue(attributesMissingErrorScreen.attributesMissingMessageVisible(), "Error screen is not visible")
    }
}
