package feature.personalize

import helper.IssuanceDataHelper
import helper.LocalizationHelper
import helper.OrganizationAuthMetadataHelper
import helper.TasDataHelper
import helper.TestBase
import navigator.MenuNavigator
import navigator.OnboardingNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertAll
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.card.CardDetailScreen
import screen.dashboard.DashboardScreen
import screen.history.HistoryOverviewScreen
import screen.menu.MenuScreen
import screen.personalize.CardIssuanceScreen
import screen.personalize.DisclosureIssuanceScreen
import screen.security.PinScreen
import screen.web.demo.DemoIndexWebPage
import screen.web.demo.issuer.IssuerWebPage

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${PersonalizeAppHandlesDigidAuthenticationTests.USE_CASE} Renew card [${PersonalizeAppHandlesDigidAuthenticationTests.JIRA_ID}]")
class RenewCardTests : TestBase() {

    companion object {
        const val USE_CASE = "4.3"
        const val JIRA_ID = "PVW-3819"
    }

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
        indexWebPage.clickInsuranceButton()
        val platform = indexWebPage.platformName()
        issuerWebPage.openSameDeviceWalletFlow(platform)
        issuerWebPage.switchToAppContext()
        disclosureForIssuanceScreen.share()
        pinScreen.enterPin(OnboardingNavigator.PIN)
        cardIssuanceScreen.clickAddButton()
        pinScreen.enterPin(OnboardingNavigator.PIN)
        cardIssuanceScreen.clickToDashboardButton()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("Renewing an insurance card")
    fun verifyInsuranceIssuance(testInfo: TestInfo) {
        setUp(testInfo)
        dashboardScreen.clickMenuButton()
        MenuScreen().clickBrowserTestButton()
        indexWebPage.clickInsuranceButton()
        val platform = indexWebPage.platformName()
        issuerWebPage.openSameDeviceWalletFlow(platform)
        issuerWebPage.switchToAppContext()
        disclosureForIssuanceScreen.share()
        pinScreen.enterPin(OnboardingNavigator.PIN)

        assertTrue(cardIssuanceScreen.renewCardSectionTitleVisible(), "renew card screen not displayed")
        cardIssuanceScreen.clickAddButton()
        pinScreen.enterPin(OnboardingNavigator.PIN)
        cardIssuanceScreen.clickToDashboardButton()
        assertTrue(dashboardScreen.cardVisible(tasData.getInsuranceVCT()), "Insurance card not visible on dashboard")
        dashboardScreen.clickCard(tasData.getInsuranceVCT())
        cardDetailScreen.clickCardHistoryButton()
        assertAll(
            { assertTrue(historyOverviewScreen.issuanceSubtitleVisible(), "data is not visible") },
            { assertTrue(historyOverviewScreen.renewCardSubtitleVisible(), "data is not visible") },
        )
    }
}
