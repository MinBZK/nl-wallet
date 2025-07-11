package feature.personalize

import helper.IssuanceDataHelper
import helper.LocalizationHelper
import helper.OrganizationAuthMetadataHelper
import helper.OrganizationAuthMetadataHelper.Organization.INSURANCE
import helper.OrganizationAuthMetadataHelper.Organization.UNIVERSITY
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
import screen.dashboard.DashboardScreen
import screen.menu.MenuScreen
import screen.personalize.CardIssuanceScreen
import screen.personalize.DisclosureIssuanceScreen
import screen.security.PinScreen
import screen.web.demo.DemoIndexWebPage
import screen.web.demo.issuer.IssuerWebPage

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${PersonalizeAppHandlesDigidAuthenticationTests.USE_CASE} Disclosure Based Issuance [${PersonalizeAppHandlesDigidAuthenticationTests.JIRA_ID}]")
class DisclosureBasedIssuanceTests : TestBase() {

    companion object {
        const val USE_CASE = "disclosure based issuance"
        const val JIRA_ID = "PVW-3799"
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
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("Adding a diploma")
    fun verifyDiplomaIssuance(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        indexWebPage.clickHollandUniversityButton()
        val platform = indexWebPage.platformName()
        issuerWebPage.openSameDeviceWalletFlow(platform)
        issuerWebPage.switchToAppContext()
        assertTrue(disclosureForIssuanceScreen.organizationNameVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", UNIVERSITY)))
        disclosureForIssuanceScreen.viewDetails()
        assertTrue(disclosureForIssuanceScreen.requestedAttributeVisible(tasData.getPidClaimLabel("bsn")))
        disclosureForIssuanceScreen.goBack();
        disclosureForIssuanceScreen.share()
        pinScreen.enterPin(OnboardingNavigator.PIN)
        cardIssuanceScreen.viewDetails()

        assertAll(
            { assertTrue(cardIssuanceScreen.organizationInSubtitleVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", UNIVERSITY)), "Subtitle is not visible") },
            { assertTrue(cardIssuanceScreen.labelVisible(tasData.getDiplomaClaimLabel("graduation_date")), "Label is not visible") },
            { assertTrue(cardIssuanceScreen.labelVisible(tasData.getDiplomaClaimLabel("grade")), "Label is not visible") },
            { assertTrue(cardIssuanceScreen.dataVisible(l10n.getString("cardValueNull")), "data is not visible") },
            { assertTrue(cardIssuanceScreen.dataVisible(issuanceData.getAttributeValues("university", "999991772", "university").first()), "data is not visible") },
            { assertTrue(cardIssuanceScreen.dataVisible(issuanceData.getAttributeValues("university", "999991772", "education").first()), "data is not visible") },
        )
        cardIssuanceScreen.clickBackButton()
        cardIssuanceScreen.clickAddButton()
        pinScreen.enterPin(OnboardingNavigator.PIN)
        cardIssuanceScreen.clickToDashboardButton()
        assertTrue(dashboardScreen.cardVisible(tasData.getDiplomaVCT()), "Diploma card not visible on dashboard")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("Adding an insurance")
    fun verifyInsuranceIssuance(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        indexWebPage.clickInsurAnceButton()
        val platform = indexWebPage.platformName()
        issuerWebPage.openSameDeviceWalletFlow(platform)
        issuerWebPage.switchToAppContext()
        assertTrue(disclosureForIssuanceScreen.organizationNameVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", INSURANCE)))
        disclosureForIssuanceScreen.viewDetails()
        assertTrue(disclosureForIssuanceScreen.requestedAttributeVisible(tasData.getPidClaimLabel("bsn")))
        disclosureForIssuanceScreen.goBack();
        disclosureForIssuanceScreen.share()
        pinScreen.enterPin(OnboardingNavigator.PIN)
        cardIssuanceScreen.viewDetails()
        assertAll(
            { assertTrue(cardIssuanceScreen.organizationInSubtitleVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", INSURANCE)), "Subtitle is not visible") },
            { assertTrue(cardIssuanceScreen.labelVisible(tasData.getInsuranceClaimLabel("start_date")), "Label is not visible") },
            { assertTrue(cardIssuanceScreen.labelVisible(tasData.getInsuranceClaimLabel("duration")), "Label is not visible") },
            { assertTrue(cardIssuanceScreen.dataVisible(issuanceData.getAttributeValues("insurance", "999991772", "product").first()), "data is not visible") },
            { assertTrue(cardIssuanceScreen.dataVisible(issuanceData.getAttributeValues("insurance", "999991772", "coverage").first()), "data is not visible") },
        )
        cardIssuanceScreen.clickBackButton()
        cardIssuanceScreen.clickAddButton()
        pinScreen.enterPin(OnboardingNavigator.PIN)
        cardIssuanceScreen.clickToDashboardButton();
        assertTrue(dashboardScreen.cardVisible(tasData.getInsuranceVCT()), "insurance card not visible on dashboard")
    }
}
