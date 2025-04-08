package feature.history

import helper.LocalizationHelper
import helper.LocalizationHelper.Translation.AMSTERDAM_DISPLAY_NAME
import helper.LocalizationHelper.Translation.BIRTH_DATE_LABEL
import helper.LocalizationHelper.Translation.BSN_LABEL
import helper.LocalizationHelper.Translation.FIRST_NAME_LABEL
import helper.LocalizationHelper.Translation.NAME_LABEL
import helper.LocalizationHelper.Translation.OVER_18_LABEL
import helper.LocalizationHelper.Translation.PID_CARD_TITLE
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
    fun verifyIssuanceHistoryEntries(testInfo: TestInfo) {
        setUp(testInfo)

        val historyOverviewScreen = HistoryOverviewScreen()
        historyOverviewScreen.loginDisclosureOrganizationVisible()
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
        val l10n = LocalizationHelper()
        assertAll(
            { assertTrue(historyDetailScreen.issuanceOrganizationVisible("RvIG"), "organization not visible") },
            { assertTrue(historyDetailScreen.titleCorrectForIssuance(l10n.translate(PID_CARD_TITLE)), "title not visible") }
        )

        // 1. The identity of party the interaction was done with (e.g. issuer or verifier) with a link to its detailed information, moving to PVW-1997.
        historyDetailScreen.openOrganizationScreen();
        val organizationDetailScreen = OrganizationDetailScreen()
        assertTrue(organizationDetailScreen.organizationInHeaderVisible("RvIG"), "organization not visible")
        organizationDetailScreen.clickBackButton()
        // 3. All the attributes that were issued or disclosed, grouped per card, including their values.
        // 1(3.)  The page offers an entrance to Report a Problem
        assertAll(
            { assertTrue(historyDetailScreen.attributeLabelVisible(l10n.translate(NAME_LABEL)), "lastname label not visible") },
            { assertTrue(historyDetailScreen.attributeLabelVisible(l10n.translate(FIRST_NAME_LABEL)), "firts name label not visible") },
            { assertTrue(historyDetailScreen.attributeLabelVisible(l10n.translate(BIRTH_DATE_LABEL)), "birthdate label not visible") },
            { assertTrue(historyDetailScreen.attributeLabelVisible(l10n.translate(OVER_18_LABEL)), "18+ label not visible") },
            { assertTrue(historyDetailScreen.attributeLabelVisible(l10n.translate(BSN_LABEL)), "BSN label not visible") },
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
            { assertTrue(historyDetailScreen.attributeLabelVisible(l10n.translate(BSN_LABEL)), "BSN label not visible") },
            { assertTrue(historyDetailScreen.disclosureOrganizationVisible(l10n.translate(AMSTERDAM_DISPLAY_NAME)), "organization not visible") },
            { assertTrue(historyDetailScreen.titleCorrectForLogin(l10n.translate(AMSTERDAM_DISPLAY_NAME)), "title not visible") },
            { assertTrue(historyDetailScreen.reportProblemButtonVisible(), "report problem button not visible") },
            { assertTrue(historyDetailScreen.termsVisible(), "terms not visible") }
        )
    }
    /**
     * 2. A log entry is only stored locally on the smartphone.
     * >> This requirement hard, if not impossible to be tested in an e2e setup and should be validated during an audit of the app.
     */
}
