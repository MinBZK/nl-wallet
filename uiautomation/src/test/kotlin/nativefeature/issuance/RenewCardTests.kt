package nativefeature.issuance

import helper.IssuanceDataHelper
import helper.LocalizationHelper
import helper.OrganizationAuthMetadataHelper
import helper.TasDataHelper
import helper.TestBase
import nativenavigator.MenuNavigator
import nativenavigator.screen.MenuNavigatorScreen
import nativescreen.card.CardDetailScreen
import nativescreen.dashboard.DashboardScreen
import nativescreen.history.HistoryOverviewScreen
import nativescreen.issuance.CardIssuanceScreen
import nativescreen.issuance.DisclosureIssuanceScreen
import nativescreen.menu.MenuScreen
import nativescreen.security.PinScreen
import nativescreen.web.demo.DemoIndexWebPage
import nativescreen.web.demo.issuer.IssuerWebPage
import org.junit.jupiter.api.Assertions.assertAll
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("Use Case 4.1 Obtain one or more cards from a (Q)EAA Issuer")
class RenewCardTests : TestBase() {

    private lateinit var indexWebPage: DemoIndexWebPage
    private lateinit var issuerWebPage: IssuerWebPage
    private lateinit var disclosureForIssuanceScreen: DisclosureIssuanceScreen
    private lateinit var cardIssuanceScreen: CardIssuanceScreen
    private lateinit var pinScreen: PinScreen
    private lateinit var l10n: LocalizationHelper
    private lateinit var tasData: TasDataHelper
    private lateinit var issuanceData : IssuanceDataHelper
    private lateinit var organizationAuthMetadata: OrganizationAuthMetadataHelper
    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var cardDetailScreen: CardDetailScreen
    private lateinit var historyOverviewScreen: HistoryOverviewScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        indexWebPage = DemoIndexWebPage()
        issuerWebPage = IssuerWebPage()
        pinScreen = PinScreen()
        l10n = LocalizationHelper()
        tasData = TasDataHelper()
        organizationAuthMetadata = OrganizationAuthMetadataHelper()
        disclosureForIssuanceScreen = DisclosureIssuanceScreen()
        cardIssuanceScreen = CardIssuanceScreen()
        dashboardScreen = DashboardScreen()
        issuanceData = IssuanceDataHelper()
        cardDetailScreen = CardDetailScreen()
        historyOverviewScreen = HistoryOverviewScreen()

        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        indexWebPage.switchToWebViewContext()
        indexWebPage.clickInsuranceButton()
        issuerWebPage.openSameDeviceWalletFlow()
        disclosureForIssuanceScreen.switchToNativeContext()
        disclosureForIssuanceScreen.share()
        pinScreen.enterPin(DEFAULT_PIN)
        cardIssuanceScreen.clickAddButton()
        pinScreen.enterPin(DEFAULT_PIN)
        cardIssuanceScreen.clickToDashboardButton()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC12 Renewing a card")
    fun verifyInsuranceIssuance(testInfo: TestInfo) {
        setUp(testInfo)
        dashboardScreen.clickMenuButton()
        MenuScreen().clickBrowserTestButton()
        indexWebPage.switchToWebViewContext()
        indexWebPage.clickInsuranceButton()
        issuerWebPage.openSameDeviceWalletFlow()
        issuerWebPage.switchToNativeContext()
        disclosureForIssuanceScreen.share()
        pinScreen.enterPin(DEFAULT_PIN)
        assertTrue(cardIssuanceScreen.renewCardSectionTitleVisible(), "renew card screen not displayed")

        cardIssuanceScreen.clickAddButton()
        pinScreen.enterPin(DEFAULT_PIN)
        cardIssuanceScreen.clickToDashboardButton()
        dashboardScreen.scrollToEndOfScreen()
        assertTrue(dashboardScreen.cardVisible(tasData.getInsuranceDisplayName()), "Insurance card not visible on dashboard")

        dashboardScreen.clickCard(tasData.getInsuranceDisplayName())
        cardDetailScreen.clickCardHistoryButton()
        assertAll(
            { assertTrue(historyOverviewScreen.issuanceSubtitleVisible(), "data is not visible") },
            { assertTrue(historyOverviewScreen.renewCardSubtitleVisible(), "data is not visible") },
        )
    }
}
