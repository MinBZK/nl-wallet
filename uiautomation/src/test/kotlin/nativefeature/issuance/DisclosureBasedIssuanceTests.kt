package nativefeature.issuance

import helper.IssuanceDataHelper
import helper.LocalizationHelper
import helper.OrganizationAuthMetadataHelper
import helper.OrganizationAuthMetadataHelper.Organization.INSURANCE
import helper.OrganizationAuthMetadataHelper.Organization.UNIVERSITY
import helper.TasDataHelper
import helper.TestBase
import nativenavigator.MenuNavigator
import nativenavigator.screen.MenuNavigatorScreen
import nativescreen.dashboard.DashboardScreen
import nativescreen.error.NoCardsErrorScreen
import nativescreen.issuance.CardIssuanceScreen
import nativescreen.issuance.DisclosureIssuanceScreen
import nativescreen.menu.MenuScreen
import nativescreen.security.PinScreen
import nativescreen.web.demo.DemoIndexWebPage
import nativescreen.web.demo.issuer.IssuerWebPage
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junit.jupiter.api.assertAll
import org.junitpioneer.jupiter.RetryingTest

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("Use Case 4.1 Obtain one or more cards from a (Q)EAA Issuer")
class DisclosureBasedIssuanceTests : TestBase() {

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
    private lateinit var noCardsErrorScreen: NoCardsErrorScreen

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
        noCardsErrorScreen = NoCardsErrorScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC5 Disclosure based Issuance happy flow, university")
    fun verifyDiplomaIssuance(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        indexWebPage.switchToWebViewContext()
        indexWebPage.clickHollandUniversityButton()
        issuerWebPage.openSameDeviceWalletFlow()

        disclosureForIssuanceScreen.switchToNativeContext()
        assertTrue(disclosureForIssuanceScreen.organizationNameVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", UNIVERSITY)))

        disclosureForIssuanceScreen.viewDetails()
        assertTrue(disclosureForIssuanceScreen.requestedAttributeVisible(tasData.getPidClaimLabel("bsn")))

        disclosureForIssuanceScreen.goBack();
        disclosureForIssuanceScreen.share()
        pinScreen.enterPin(DEFAULT_PIN)
        cardIssuanceScreen.viewDetails()
        assertAll(
            { assertTrue(cardIssuanceScreen.organizationInSubtitleVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", UNIVERSITY)), "Subtitle is not visible") },
            { assertTrue(cardIssuanceScreen.labelVisible(tasData.getDiplomaClaimLabel("graduation_date")), "Label is not visible") },
            { assertTrue(cardIssuanceScreen.labelVisible(tasData.getDiplomaClaimLabel("grade")), "Label is not visible") },
            { assertTrue(cardIssuanceScreen.dataVisible(l10n.getString("cardValueNull")), "data is not visible") },
            { assertTrue(cardIssuanceScreen.dataVisible(issuanceData.getAttributeValues("university", DEFAULT_BSN, "university").first()), "data is not visible") },
            { assertTrue(cardIssuanceScreen.dataVisible(issuanceData.getAttributeValues("university", DEFAULT_BSN, "education").first()), "data is not visible") },
        )
        cardIssuanceScreen.clickBackButton()
        cardIssuanceScreen.clickAddButton()
        pinScreen.enterPin(DEFAULT_PIN)
        cardIssuanceScreen.clickToDashboardButton()
        dashboardScreen.scrollToEndOfScreen()
        assertTrue(dashboardScreen.cardVisible(tasData.getDiplomaDisplayName()), "Diploma card not visible on dashboard")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC5 Disclosure based Issuance happy flow, insurance")
    fun verifyInsuranceIssuance(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        indexWebPage.switchToWebViewContext()
        indexWebPage.clickInsuranceButton()
        issuerWebPage.openSameDeviceWalletFlow()

        disclosureForIssuanceScreen.switchToNativeContext()
        assertTrue(disclosureForIssuanceScreen.organizationNameVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", INSURANCE)))

        disclosureForIssuanceScreen.viewDetails()
        assertTrue(disclosureForIssuanceScreen.requestedAttributeVisible(tasData.getPidClaimLabel("bsn")))

        disclosureForIssuanceScreen.goBack();
        disclosureForIssuanceScreen.share()
        pinScreen.enterPin(DEFAULT_PIN)
        cardIssuanceScreen.viewDetails()
        assertAll(
            { assertTrue(cardIssuanceScreen.organizationInSubtitleVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", INSURANCE)), "Subtitle is not visible") },
            { assertTrue(cardIssuanceScreen.labelVisible(tasData.getInsuranceClaimLabel("start_date")), "Label is not visible") },
            { assertTrue(cardIssuanceScreen.labelVisible(tasData.getInsuranceClaimLabel("duration")), "Label is not visible") },
            { assertTrue(cardIssuanceScreen.dataVisible(issuanceData.getAttributeValues("insurance", DEFAULT_BSN, "product").first()), "data is not visible") },
            { assertTrue(cardIssuanceScreen.dataVisible(issuanceData.getAttributeValues("insurance", DEFAULT_BSN, "coverage").first()), "data is not visible") },
        )

        cardIssuanceScreen.clickBackButton()
        cardIssuanceScreen.clickAddButton()
        pinScreen.enterPin(DEFAULT_PIN)
        cardIssuanceScreen.clickToDashboardButton();
        dashboardScreen.scrollToEndOfScreen()
        assertTrue(dashboardScreen.cardVisible(tasData.getInsuranceDisplayName()), "insurance card not visible on dashboard")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC79 No cards to be issued")
    fun verifyNoInsuranceCardAvailable(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu, "900265462")
        MenuScreen().clickBrowserTestButton()
        indexWebPage.switchToWebViewContext()
        indexWebPage.clickInsuranceButton()
        issuerWebPage.openSameDeviceWalletFlow()

        disclosureForIssuanceScreen.switchToNativeContext()
        disclosureForIssuanceScreen.share()
        pinScreen.enterPin(DEFAULT_PIN)
        assertTrue(noCardsErrorScreen.titleVisible(), "no card error screen is not visible")

        noCardsErrorScreen.close()
        assertTrue(dashboardScreen.cardVisible(tasData.getPidDisplayName()), "Pid not visible on dashboard")
    }
}
