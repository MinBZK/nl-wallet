package feature.issuance

import helper.GbaDataHelper
import helper.GbaDataHelper.Field.FIRST_NAME
import helper.GbaDataHelper.Field.NAME
import helper.IssuanceDataHelper
import helper.LocalizationHelper
import helper.OrganizationMetadataHelper
import helper.TasDataHelper
import helper.TestBase
import navigator.MenuNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertAll
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.card.CardDetailScreen
import screen.dashboard.DashboardScreen
import screen.history.HistoryOverviewScreen
import screen.issuance.CardIssuanceScreen
import screen.issuance.DisclosureIssuanceScreen
import screen.issuance.PersonalizeInformScreen
import screen.issuance.PersonalizePidPreviewScreen
import screen.menu.MenuScreen
import screen.security.PinScreen
import screen.web.demo.DemoIndexWebPage
import screen.web.demo.issuer.IssuerWebPage
import screen.web.digid.DigidLoginMockWebPage

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("Use Case 4.1 Obtain one or more cards from a (Q)EAA Issuer")
class RenewCardTests : TestBase() {

    private lateinit var l10n: LocalizationHelper
    private lateinit var tasData: TasDataHelper
    private lateinit var issuanceData : IssuanceDataHelper
    private lateinit var gbaData: GbaDataHelper

    private lateinit var indexWebPage: DemoIndexWebPage
    private lateinit var issuerWebPage: IssuerWebPage
    private lateinit var disclosureForIssuanceScreen: DisclosureIssuanceScreen
    private lateinit var cardIssuanceScreen: CardIssuanceScreen
    private lateinit var pinScreen: PinScreen
    private lateinit var organizationAuthMetadata: OrganizationMetadataHelper
    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var cardDetailScreen: CardDetailScreen
    private lateinit var historyOverviewScreen: HistoryOverviewScreen
    private lateinit var personalizeInformScreen: PersonalizeInformScreen
    private lateinit var digidLoginMockWebPage: DigidLoginMockWebPage
    private lateinit var personalizePidPreviewScreen: PersonalizePidPreviewScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)

        l10n = LocalizationHelper()
        tasData = TasDataHelper()
        gbaData = GbaDataHelper()
        issuanceData = IssuanceDataHelper()

        indexWebPage = DemoIndexWebPage()
        issuerWebPage = IssuerWebPage()
        pinScreen = PinScreen()
        organizationAuthMetadata = OrganizationMetadataHelper()
        disclosureForIssuanceScreen = DisclosureIssuanceScreen()
        cardIssuanceScreen = CardIssuanceScreen()
        dashboardScreen = DashboardScreen()
        cardDetailScreen = CardDetailScreen()
        historyOverviewScreen = HistoryOverviewScreen()
        personalizeInformScreen = PersonalizeInformScreen()
        digidLoginMockWebPage = DigidLoginMockWebPage()
        personalizePidPreviewScreen = PersonalizePidPreviewScreen()

        MenuNavigator().toScreen(MenuNavigatorScreen.Dashboard)

    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC11 Renew card")
    fun verifyMdocDiplomaIssuance(testInfo: TestInfo) {
        setUp(testInfo)
        dashboardScreen.clickMenuButton()
        MenuScreen().clickBrowserTestButton()
        indexWebPage.switchToWebViewContext()
        indexWebPage.clickHollandUniversityMdocButton()
        issuerWebPage.openSameDeviceWalletFlow()
        disclosureForIssuanceScreen.switchToNativeContext()
        disclosureForIssuanceScreen.share()
        pinScreen.enterPin(DEFAULT_PIN)
        cardIssuanceScreen.clickAdd2CardsButton()
        pinScreen.enterPin(DEFAULT_PIN)
        cardIssuanceScreen.clickToDashboardButton()

        dashboardScreen.clickMenuButton()
        MenuScreen().clickBrowserTestButton()
        indexWebPage.switchToWebViewContext()
        indexWebPage.clickHollandUniversityMdocButton()
        issuerWebPage.openSameDeviceWalletFlow()
        issuerWebPage.switchToNativeContext()
        disclosureForIssuanceScreen.share()
        pinScreen.enterPin(DEFAULT_PIN)
        assertTrue(cardIssuanceScreen.renewCardsSectionTitleVisible(), "renew cards screen not displayed")

        cardIssuanceScreen.clickAdd2CardsButton()
        pinScreen.enterPin(DEFAULT_PIN)
        cardIssuanceScreen.clickToDashboardButton()
        dashboardScreen.scrollToEndOfScreen()
        assertTrue(dashboardScreen.cardVisible(tasData.getDiplomaDisplayName()), "Diploma card not visible on dashboard")

        dashboardScreen.clickCard(tasData.getDiplomaDisplayName())
        cardDetailScreen.clickCardHistoryButton()
        assertAll(
            { assertTrue(historyOverviewScreen.issuanceSubtitleVisible(), "data is not visible") },
            { assertTrue(historyOverviewScreen.renewCardSubtitleVisible(), "data is not visible") },
        )
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC66 Renew PID")
    @Tags(Tag("a11yBatch3"))
    fun verifyPIDCardRenewal(testInfo: TestInfo) {
        setUp(testInfo)
        dashboardScreen.clickCard(tasData.getPidDisplayName())
        cardDetailScreen.renewPidCard()
        personalizeInformScreen.clickDigidLoginButton()
        digidLoginMockWebPage.switchToBrowser()
        digidLoginMockWebPage.switchToWebViewContext()
        digidLoginMockWebPage.login(DEFAULT_BSN)
        personalizePidPreviewScreen.switchToNativeContext()
        assertAll(
            { assertTrue(personalizePidPreviewScreen.renewPidCardTitleVisible(), "Renew pid preview screen is not visible") },
            { assertTrue(personalizePidPreviewScreen.humanReadableCardDataVisible(gbaData.getValueByField(FIRST_NAME, DEFAULT_BSN)), "human readable pid data is not visible") },
            { assertTrue(personalizePidPreviewScreen.humanReadableCardDataVisible(gbaData.getValueByField(NAME, DEFAULT_BSN)), "human readable pid data is not visible") },
        )

        personalizePidPreviewScreen.clickAcceptPidRenewalButton()
        pinScreen.enterPin(DEFAULT_PIN)
        cardIssuanceScreen.clickToDashboardButton()
        dashboardScreen.clickCard(tasData.getPidDisplayName())

        cardDetailScreen.clickCardHistoryButton()
        assertTrue(historyOverviewScreen.renewCardSubtitleVisible(), "Renew PID history event not visible")
    }
}
