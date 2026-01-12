package feature.issuance

import helper.IssuanceDataHelper
import helper.LocalizationHelper
import helper.OrganizationAuthMetadataHelper
import helper.OrganizationAuthMetadataHelper.Organization.INSURANCE
import helper.OrganizationAuthMetadataHelper.Organization.UNIVERSITY
import helper.TasDataHelper
import helper.TestBase
import navigator.MenuNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junit.jupiter.api.assertAll
import org.junitpioneer.jupiter.RetryingTest
import screen.dashboard.DashboardScreen
import screen.disclosure.SharingStoppedScreen
import screen.error.NoCardsErrorScreen
import screen.issuance.CardIssuanceScreen
import screen.issuance.DisclosureIssuanceScreen
import screen.menu.MenuScreen
import screen.security.PinScreen
import screen.web.demo.DemoIndexWebPage
import screen.web.demo.issuer.IssuerWebPage

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
    private lateinit var sharingStoppedScreen: SharingStoppedScreen

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
        sharingStoppedScreen = SharingStoppedScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC5 Disclosure based Issuance happy flow, university, SD-JWT")
    @Tags(Tag("a11yBatch3"))
    fun verifyDiplomaIssuanceSdJwt(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        indexWebPage.switchToWebViewContext()
        indexWebPage.clickHollandUniversitySdJwtButton()
        issuerWebPage.openSameDeviceWalletFlow()

        disclosureForIssuanceScreen.switchToNativeContext()
        assertTrue(disclosureForIssuanceScreen.organizationNameVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", UNIVERSITY)))

        disclosureForIssuanceScreen.viewDetails()
        assertTrue(disclosureForIssuanceScreen.requestedAttributeVisible(tasData.getPidClaimLabel("bsn")))

        disclosureForIssuanceScreen.goBack();
        disclosureForIssuanceScreen.share()
        pinScreen.enterPin(DEFAULT_PIN)
        cardIssuanceScreen.viewDetailsOfCard(issuanceData.getAttributeValues("university", DEFAULT_BSN, "education").last())
        assertAll(
            { assertTrue(cardIssuanceScreen.organizationInSubtitleVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", UNIVERSITY)), "Subtitle is not visible") },
            { assertTrue(cardIssuanceScreen.labelVisible(tasData.getDiplomaClaimLabel("graduation_date")), "Label is not visible") },
            { assertTrue(cardIssuanceScreen.labelVisible(tasData.getDiplomaClaimLabel("grade")), "Label is not visible") },
            { assertTrue(cardIssuanceScreen.dataVisible(issuanceData.getAttributeValues("university", DEFAULT_BSN, "university").last()), "data is not visible") },
            { assertTrue(cardIssuanceScreen.dataVisible(issuanceData.getAttributeValues("university", DEFAULT_BSN, "education").last()), "data is not visible") },
        )

        cardIssuanceScreen.clickBackButton()
        cardIssuanceScreen.clickAdd2CardsButton()
        pinScreen.enterPin(DEFAULT_PIN)

        cardIssuanceScreen.clickToDashboardButton()
        dashboardScreen.scrollToEndOfScreen()
        assertTrue(dashboardScreen.cardVisible(tasData.getDiplomaDisplayName()), "Diploma card not visible on dashboard")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC5 Disclosure based Issuance happy flow, university, MDOC")
    fun verifyDiplomaIssuanceMdoc(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        indexWebPage.switchToWebViewContext()
        indexWebPage.clickHollandUniversityMdocButton()
        issuerWebPage.openSameDeviceWalletFlow()

        disclosureForIssuanceScreen.switchToNativeContext()
        assertTrue(disclosureForIssuanceScreen.organizationNameVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", UNIVERSITY)))

        disclosureForIssuanceScreen.viewDetails()
        assertTrue(disclosureForIssuanceScreen.requestedAttributeVisible(tasData.getPidClaimLabel("bsn")))

        disclosureForIssuanceScreen.goBack();
        disclosureForIssuanceScreen.share()
        pinScreen.enterPin(DEFAULT_PIN)
        cardIssuanceScreen.viewDetailsOfCard(issuanceData.getAttributeValues("university", DEFAULT_BSN, "education").last())
        assertAll(
            { assertTrue(cardIssuanceScreen.organizationInSubtitleVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", UNIVERSITY)), "Subtitle is not visible") },
            { assertTrue(cardIssuanceScreen.labelVisible(tasData.getDiplomaClaimLabel("graduation_date")), "Label is not visible") },
            { assertTrue(cardIssuanceScreen.labelVisible(tasData.getDiplomaClaimLabel("grade")), "Label is not visible") },
            { assertTrue(cardIssuanceScreen.dataVisible(issuanceData.getAttributeValues("university", DEFAULT_BSN, "grade").last()), "data is not visible") },
            { assertTrue(cardIssuanceScreen.dataVisible(issuanceData.getAttributeValues("university", DEFAULT_BSN, "university").last()), "data is not visible") },
            { assertTrue(cardIssuanceScreen.dataVisible(issuanceData.getAttributeValues("university", DEFAULT_BSN, "education").last()), "data is not visible") },
        )
        cardIssuanceScreen.clickBackButton()
        cardIssuanceScreen.clickAdd2CardsButton()
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
        cardIssuanceScreen.viewDetailsOfCard(issuanceData.getAttributeValues("insurance", DEFAULT_BSN, "coverage").first())
        assertAll(
            { assertTrue(cardIssuanceScreen.organizationInSubtitleVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", INSURANCE)), "Subtitle is not visible") },
            { assertTrue(cardIssuanceScreen.labelVisible(tasData.getInsuranceClaimLabel("start_date")), "Label is not visible") },
            { assertTrue(cardIssuanceScreen.labelVisible(tasData.getInsuranceClaimLabel("duration")), "Label is not visible") },
            { assertTrue(cardIssuanceScreen.dataVisible(issuanceData.getAttributeValues("insurance", DEFAULT_BSN, "product").first()), "data is not visible") },
            { assertTrue(cardIssuanceScreen.dataVisible(issuanceData.getAttributeValues("insurance", DEFAULT_BSN, "coverage").first()), "data is not visible") },
        )

        cardIssuanceScreen.clickBackButton()
        cardIssuanceScreen.clickAddCardButton()
        pinScreen.enterPin(DEFAULT_PIN)
        cardIssuanceScreen.clickToDashboardButton();
        dashboardScreen.scrollToEndOfScreen()
        assertAll(
            { assertTrue(dashboardScreen.cardVisible(tasData.getInsuranceDisplayName()), "insurance card not visible on dashboard") },
            { assertTrue(dashboardScreen.checkCardSorting(tasData.getPidDisplayName(), tasData.getInsuranceDisplayName()), "Card sorting is incorrect") },
        )
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC9 No cards to be issued")
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

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC8 Reject disclosure of attributes")
    fun verifyRejectDisclosureOfAttributes(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        indexWebPage.switchToWebViewContext()
        indexWebPage.clickInsuranceButton()
        issuerWebPage.openSameDeviceWalletFlow()
        disclosureForIssuanceScreen.switchToNativeContext()
        disclosureForIssuanceScreen.stop()
        disclosureForIssuanceScreen.bottomSheetConfirmStop();
        assertAll (
            { assertTrue(sharingStoppedScreen.titleVisible(), "Title is not visible") },
            { assertTrue(sharingStoppedScreen.descriptionVisible(), "Description is not visible") },
        )

        sharingStoppedScreen.close();
        assertTrue(dashboardScreen.visible(), "Dashboard not visible")
    }
}
