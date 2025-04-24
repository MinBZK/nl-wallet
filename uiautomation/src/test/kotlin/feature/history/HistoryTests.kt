package feature.history

import helper.CardMetadataHelper
import helper.LocalizationHelper
import helper.OrganizationAuthMetadataHelper
import helper.OrganizationAuthMetadataHelper.Organization.AMSTERDAM
import helper.OrganizationAuthMetadataHelper.Organization.RVIG
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
import screen.dashboard.DashboardScreen
import screen.disclosure.DisclosureApproveOrganizationScreen
import screen.history.HistoryDetailScreen
import screen.history.HistoryOverviewScreen
import screen.menu.MenuScreen
import screen.organization.OrganizationDetailScreen
import screen.security.PinScreen
import screen.web.rp.RelyingPartyAmsterdamWebPage
import screen.web.rp.RelyingPartyOverviewWebPage

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${HistoryTests.USE_CASE} App logs PID/address issuance and disclosure events [${HistoryTests.JIRA_ID}]")
class HistoryTests : TestBase() {

    // Tests in these class also verify the functionality in: PVW-1231 As a User, I want to see the all
    // relevant details of a particular event that occurred with my wallet for better understanding of how my data is used
    companion object {
        const val USE_CASE = "UC 3.1"
        const val JIRA_ID = "PVW-1038"
    }

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()

        val overviewWebPage = RelyingPartyOverviewWebPage()
        val platform = overviewWebPage.platformName()

        overviewWebPage.clickAmsterdamButton()
        val amsterdamWebPage = RelyingPartyAmsterdamWebPage()
        amsterdamWebPage.openSameDeviceWalletFlow(platform)
        amsterdamWebPage.switchToAppContext()

        val disclosureScreen = DisclosureApproveOrganizationScreen()
        disclosureScreen.login()
        PinScreen().enterPin(OnboardingNavigator.PIN)

        disclosureScreen.closeDialog()
        DashboardScreen().clickMenuButton()
        MenuScreen().clickHistoryButton()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.A log entries are added for the PID issuance and disclosure events. [${JIRA_ID}]")
    fun verifyHistoryEntries(testInfo: TestInfo) {
        setUp(testInfo)
        val l10n = LocalizationHelper()
        val pidData = CardMetadataHelper()
        val organizationAuthData = OrganizationAuthMetadataHelper()

        val historyOverviewScreen = HistoryOverviewScreen()
        historyOverviewScreen.disclosureOrganizationVisible(organizationAuthData.getAttributeValueForOrganization("organization.displayName", AMSTERDAM))
        assertAll(
            { assertTrue(historyOverviewScreen.visible(), "history overview screen is not visible") },
            { assertTrue(historyOverviewScreen.pidIssuanceLogEntryVisible(), "pid issuance log entry is not visible") },
            { assertTrue(historyOverviewScreen.issuanceSubtitleVisible(), "pid issuance log entry is not visible") },
            { assertTrue(historyOverviewScreen.addressIssuanceLogEntryVisible(), "address issuance log entry is not visible") },
            { assertTrue(historyOverviewScreen.loginDisclosureLogEntryVisible(), "login log entry is not visible") }
        )

        historyOverviewScreen.clickPidCardTitle()
        // PVW-1231 2. The page (history detail) displays the following information:
        val historyDetailScreen = HistoryDetailScreen()
        assertAll(
            { assertTrue(historyDetailScreen.issuanceOrganizationVisible(organizationAuthData.getAttributeValueForOrganization("organization.displayName", RVIG)), "organization not visible") },
            { assertTrue(historyDetailScreen.titleCorrectForIssuance(pidData.getPidDisplayName()), "title not visible") }
        )

        // 1. The identity of party the interaction was done with (e.g. issuer or verifier) with a link to its detailed information, moving to PVW-1997.
        historyDetailScreen.openOrganizationScreen();
        val organizationDetailScreen = OrganizationDetailScreen()
        assertTrue(organizationDetailScreen.organizationInHeaderVisible(organizationAuthData.getAttributeValueForOrganization("organization.displayName", RVIG)), "organization not visible")
        organizationDetailScreen.clickBackButton()
        // 3. All the attributes that were issued or disclosed, grouped per card, including their values.
        // 1(3.)  The page offers an entrance to Report a Problem
        assertAll(
            { assertTrue(historyDetailScreen.attributeLabelVisible(pidData.getPidClaimLabel("family_name")), "lastname label not visible") },
            { assertTrue(historyDetailScreen.attributeLabelVisible(pidData.getPidClaimLabel("given_name")), "first name label not visible") },
            { assertTrue(historyDetailScreen.attributeLabelVisible(pidData.getPidClaimLabel("birthdate")), "birthdate label not visible") },
            { assertTrue(historyDetailScreen.attributeLabelVisible(pidData.getPidClaimLabel("age_over_18")), "18+ label not visible") },
            { assertTrue(historyDetailScreen.attributeLabelVisible(pidData.getPidClaimLabel("bsn")), "BSN label not visible") },
            { assertTrue(historyDetailScreen.reportProblemButtonVisible(), "report problem button not visible") }
        )

        // PVW-1231 1. The App offers an entrance to go back to the page it came from (full history or card history)
        historyDetailScreen.clickBottomBackButton()

        historyOverviewScreen.clickLoginEntryTitle()
        // 2. The reason for sharing (indicated by the issuer)
        // 5. The terms and conditions applicable to the transaction.
        assertAll(
            { assertTrue(historyDetailScreen.reasonForSharingHeaderVisible(), "reason for sharing header not visible") },
            { assertTrue(historyDetailScreen.reasonForSharingVisible(l10n.getString("organizationApprovePageLoginCta")), "reason for sharing not visible") },
            { assertTrue(historyDetailScreen.attributeLabelVisible(pidData.getPidClaimLabel("bsn")), "BSN label not visible") },
            { assertTrue(historyDetailScreen.disclosureOrganizationVisible(organizationAuthData.getAttributeValueForOrganization("organization.displayName", AMSTERDAM)), "organization not visible") },
            { assertTrue(historyDetailScreen.titleCorrectForLogin(organizationAuthData.getAttributeValueForOrganization("organization.displayName", AMSTERDAM)), "title not visible") },
            { assertTrue(historyDetailScreen.reportProblemButtonVisible(), "report problem button not visible") },
            { assertTrue(historyDetailScreen.termsVisible(), "terms not visible") }
        )
    }
    /**
     * 2. A log entry is only stored locally on the smartphone.
     * >> This requirement hard, if not impossible to be tested in an e2e setup and should be validated during an audit of the app.
     */
}
