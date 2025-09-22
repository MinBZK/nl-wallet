package nativefeature.cards_and_history

import helper.LocalizationHelper
import helper.OrganizationAuthMetadataHelper
import helper.OrganizationAuthMetadataHelper.Organization.AMSTERDAM
import helper.OrganizationAuthMetadataHelper.Organization.RVIG
import helper.TasDataHelper
import helper.TestBase
import nativenavigator.OnboardingNavigator
import nativenavigator.screen.OnboardingNavigatorScreen
import nativescreen.card.CardDetailScreen
import nativescreen.dashboard.DashboardScreen
import nativescreen.disclosure.DisclosureApproveOrganizationScreen
import nativescreen.history.CardHistoryScreen
import nativescreen.history.HistoryDetailScreen
import nativescreen.history.HistoryOverviewScreen
import nativescreen.menu.MenuScreen
import nativescreen.organization.OrganizationDetailScreen
import nativescreen.security.PinScreen
import nativescreen.web.demo.DemoIndexWebPage
import nativescreen.web.demo.rp.RelyingPartyAmsterdamWebPage
import org.junit.jupiter.api.Assertions.assertAll
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("UC6 Complete & card history overview and history events")
class HistoryTests : TestBase() {

    private lateinit var l10n: LocalizationHelper
    private lateinit var tasData: TasDataHelper
    private lateinit var organizationAuthData: OrganizationAuthMetadataHelper

    private lateinit var overviewWebPage: DemoIndexWebPage
    private lateinit var historyDetailScreen: HistoryDetailScreen
    private lateinit var historyOverviewScreen: HistoryOverviewScreen
    private lateinit var amsterdamWebPage: RelyingPartyAmsterdamWebPage
    private lateinit var disclosureScreen: DisclosureApproveOrganizationScreen
    private lateinit var organizationDetailScreen: OrganizationDetailScreen
    private lateinit var cardDetailScreen: CardDetailScreen
    private lateinit var cardHistoryScreen: CardHistoryScreen
    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var menuScreen: MenuScreen
    private lateinit var pinScreen: PinScreen

    fun setUp(testInfo: TestInfo) {

        startDriver(testInfo)

        l10n = LocalizationHelper()
        tasData = TasDataHelper()
        organizationAuthData = OrganizationAuthMetadataHelper()

        historyDetailScreen = HistoryDetailScreen()
        historyOverviewScreen = HistoryOverviewScreen()
        overviewWebPage = DemoIndexWebPage()
        amsterdamWebPage = RelyingPartyAmsterdamWebPage()
        disclosureScreen = DisclosureApproveOrganizationScreen()
        organizationDetailScreen = OrganizationDetailScreen()
        cardDetailScreen = CardDetailScreen()
        cardHistoryScreen = CardHistoryScreen()
        dashboardScreen = DashboardScreen()
        menuScreen = MenuScreen()
        pinScreen = PinScreen()

        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.Dashboard)
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC30 View activity list")
    fun verifyHistoryEntries(testInfo: TestInfo) {
        setUp(testInfo)
        dashboardScreen.clickMenuButton()
        menuScreen.clickBrowserTestButton()
        overviewWebPage.switchToWebViewContext()
        overviewWebPage.clickAmsterdamButton()
        amsterdamWebPage.openSameDeviceWalletFlow()

        amsterdamWebPage.switchToNativeContext()
        disclosureScreen.login()
        pinScreen.enterPin(DEFAULT_PIN)
        disclosureScreen.close()

        amsterdamWebPage.switchToWalletApp()
        amsterdamWebPage.switchToNativeContext()
        assertTrue(dashboardScreen.visible())

        dashboardScreen.clickMenuButton()
        assertTrue(menuScreen.menuListButtonsVisible())

        menuScreen.clickHistoryButton()
        assertAll(
            { assertTrue(historyOverviewScreen.visible(), "history overview screen is not visible") },
            { assertTrue(historyOverviewScreen.pidIssuanceLogEntryVisible(), "pid issuance log entry is not visible") },
            { assertTrue(historyOverviewScreen.issuanceSubtitleVisible(), "pid issuance log entry is not visible") },
            { assertTrue(historyOverviewScreen.addressIssuanceLogEntryVisible(), "address issuance log entry is not visible") },
            { assertTrue(historyOverviewScreen.loginDisclosureLogEntryVisible(), "login log entry is not visible") },
            { assertTrue(historyOverviewScreen.disclosureOrganizationVisible(organizationAuthData.getAttributeValueForOrganization("organization.displayName", AMSTERDAM))) }
        )

        historyOverviewScreen.clickPidCardTitle()
        assertAll(
            { assertTrue(historyDetailScreen.issuanceOrganizationVisible(organizationAuthData.getAttributeValueForOrganization("organization.displayName", RVIG)), "organization not visible") },
            { assertTrue(historyDetailScreen.titleCorrectForIssuance(tasData.getPidDisplayName()), "title not visible") }
        )

        historyDetailScreen.openOrganizationScreen();
        assertTrue(organizationDetailScreen.organizationInHeaderVisible(organizationAuthData.getAttributeValueForOrganization("organization.displayName", RVIG)), "organization not visible")

        organizationDetailScreen.clickBackButton()
        assertAll(
            { assertTrue(historyDetailScreen.attributeLabelVisible(tasData.getPidClaimLabel("family_name")), "lastname label not visible") },
            { assertTrue(historyDetailScreen.attributeLabelVisible(tasData.getPidClaimLabel("given_name")), "first name label not visible") },
            { assertTrue(historyDetailScreen.attributeLabelVisible(tasData.getPidClaimLabel("birthdate")), "birthdate label not visible") },
            { assertTrue(historyDetailScreen.attributeLabelVisible(tasData.getPidClaimLabel("age_over_18")), "18+ label not visible") },
            { assertTrue(historyDetailScreen.attributeLabelVisible(tasData.getPidClaimLabel("bsn")), "BSN label not visible") },
            { assertTrue(historyDetailScreen.reportProblemButtonVisible(), "report problem button not visible") }
        )

        historyDetailScreen.clickBottomBackButton()
        historyOverviewScreen.clickLoginEntryTitle()
        assertAll(
            { assertTrue(historyDetailScreen.reasonForSharingHeaderVisible(), "reason for sharing header not visible") },
            { assertTrue(historyDetailScreen.reasonForSharingVisible(l10n.getString("organizationApprovePageLoginCta")), "reason for sharing not visible") },
            { assertTrue(historyDetailScreen.attributeLabelVisible(tasData.getPidClaimLabel("bsn")), "BSN label not visible") },
            { assertTrue(historyDetailScreen.disclosureOrganizationVisible(organizationAuthData.getAttributeValueForOrganization("organization.displayName", AMSTERDAM)), "organization not visible") },
            { assertTrue(historyDetailScreen.titleCorrectForLogin(organizationAuthData.getAttributeValueForOrganization("organization.displayName", AMSTERDAM)), "title not visible") },
            { assertTrue(historyDetailScreen.reportProblemButtonVisible(), "report problem button not visible") },
            { assertTrue(historyDetailScreen.termsVisible(), "terms not visible") }
        )
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC31 View card-specific activity list")
    fun verifyCardHistory(testInfo: TestInfo) {
        setUp(testInfo)
        dashboardScreen.clickCard(tasData.getPidDisplayName())
        cardDetailScreen.clickCardHistoryButton()
        assertTrue(cardHistoryScreen.visible(), "card history screen is not visible")
    }
}
