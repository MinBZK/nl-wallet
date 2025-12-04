package feature.issuance

import helper.GbaDataHelper
import helper.GbaDataHelper.Field.CITY
import helper.GbaDataHelper.Field.FIRST_NAME
import helper.GbaDataHelper.Field.HOUSE_NUMBER
import helper.GbaDataHelper.Field.NAME
import helper.GbaDataHelper.Field.POSTAL_CODE
import helper.GbaDataHelper.Field.STREET
import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
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
import screen.error.NoInternetErrorScreen
import screen.issuance.PersonalizeAuthenticatingWithDigidScreen
import screen.issuance.PersonalizeInformScreen
import screen.issuance.PersonalizePidDataIncorrectScreen
import screen.issuance.PersonalizePidPreviewScreen
import screen.issuance.PersonalizeSuccessScreen
import screen.issuance.TransferWalletScreen
import screen.security.PinScreen
import screen.web.digid.DigidLoginMockWebPage
import screen.web.digid.DigidLoginStartWebPage

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("UC 3.1 Obtain PID")
class PidIssuanceTests : TestBase() {

    private lateinit var personalizeInformScreen: PersonalizeInformScreen
    private lateinit var personalizeAuthenticatingWithDigidScreen: PersonalizeAuthenticatingWithDigidScreen
    private lateinit var digidLoginStartWebPage: DigidLoginStartWebPage
    private lateinit var personalizePidDataIncorrectScreen: PersonalizePidDataIncorrectScreen
    private lateinit var digidLoginMockWebPage: DigidLoginMockWebPage
    private lateinit var personalizePidPreviewScreen: PersonalizePidPreviewScreen
    private lateinit var personalizeSuccessScreen: PersonalizeSuccessScreen
    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var pinScreen: PinScreen
    private lateinit var noInternetErrorScreen: NoInternetErrorScreen
    private lateinit var transferWalletScreen: TransferWalletScreen

    private lateinit var gbaData: GbaDataHelper

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.PersonalizeInform)

        personalizeInformScreen = PersonalizeInformScreen()
        personalizeAuthenticatingWithDigidScreen = PersonalizeAuthenticatingWithDigidScreen()
        digidLoginStartWebPage = DigidLoginStartWebPage()
        personalizePidDataIncorrectScreen = PersonalizePidDataIncorrectScreen()
        digidLoginMockWebPage = DigidLoginMockWebPage()
        personalizePidPreviewScreen = PersonalizePidPreviewScreen()
        personalizeSuccessScreen = PersonalizeSuccessScreen()
        pinScreen = PinScreen()
        dashboardScreen = DashboardScreen()
        noInternetErrorScreen = NoInternetErrorScreen()
        transferWalletScreen = TransferWalletScreen()
        digidLoginStartWebPage = DigidLoginStartWebPage()

        gbaData = GbaDataHelper()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC1 PID issuance happy flow")
    fun verifyPersonalizeInformScreen(testInfo: TestInfo) {
        setUp(testInfo)
        assertTrue(personalizeInformScreen.visible(), "personalize inform screen is not visible")
        assertTrue(personalizeInformScreen.digidWebsiteButtonVisible(), "digid button is not visible")

        personalizeInformScreen.clickDigidLoginButton()

        digidLoginStartWebPage.switchToWebViewContext()
        assertTrue(digidLoginStartWebPage.visible(), "digid login start web page is not visible")

        personalizeAuthenticatingWithDigidScreen.openApp()
        personalizeAuthenticatingWithDigidScreen.switchToNativeContext()
        assertAll(
            { assertTrue(personalizeAuthenticatingWithDigidScreen.awaitingUserAuthTitleVisible(), "title is not visible") },
            { assertTrue(personalizeAuthenticatingWithDigidScreen.digidLoadingStopCtaVisible(), "stop button is not visible") },
        )

        digidLoginStartWebPage.switchToBrowser()
        digidLoginStartWebPage.switchToWebViewContext()
        digidLoginStartWebPage.clickMockLoginButton()
        digidLoginMockWebPage.login(DEFAULT_BSN)

        personalizePidPreviewScreen.switchToNativeContext()
        val nationalities = gbaData.getNationalities(DEFAULT_BSN)
        assertAll(
            { assertTrue(personalizePidPreviewScreen.visible(), "personalize pid preview screen is not visible") },
            { assertTrue(personalizePidPreviewScreen.humanReadableCardDataVisible(gbaData.getValueByField(FIRST_NAME, DEFAULT_BSN)), "human readable pid data is not visible") },
            { assertTrue(personalizePidPreviewScreen.humanReadableCardDataVisible(gbaData.getValueByField(NAME, DEFAULT_BSN)), "human readable pid data is not visible") },
            { assertTrue(personalizePidPreviewScreen.humanReadableCardDataVisible(DEFAULT_BSN.toCharArray().joinToString(" ")), "human readable pid data is not visible") },
            { assertTrue(personalizePidPreviewScreen.humanReadableCardDataVisible(nationalities[0]), "array attribute is not visible") },
            { assertTrue(personalizePidPreviewScreen.humanReadableCardDataVisible(nationalities[1]), "array attribute is not visible") },
        )

        personalizePidPreviewScreen.scrollToEndOfScreen()
        assertAll(
            { assertTrue(personalizePidPreviewScreen.humanReadableCardDataVisible(gbaData.getValueByField(POSTAL_CODE, DEFAULT_BSN)), "human readable pid data is not visible") },
            { assertTrue(personalizePidPreviewScreen.humanReadableCardDataVisible(gbaData.getValueByField(STREET, DEFAULT_BSN)), "human readable pid data is not visible") },
            { assertTrue(personalizePidPreviewScreen.humanReadableCardDataVisible(gbaData.getValueByField(CITY, DEFAULT_BSN)), "human readable pid data is not visible") },
            { assertTrue(personalizePidPreviewScreen.humanReadableCardDataVisible(gbaData.getValueByField(HOUSE_NUMBER, DEFAULT_BSN)), "human readable pid data is not visible") },
            { assertTrue(personalizePidPreviewScreen.confirmButtonsVisible(), "confirm buttons are not visible") }
        )

        personalizePidPreviewScreen.clickAcceptButton()
        assertTrue(pinScreen.personalizeConfirmPinScreenVisible(), "confirm screen not visible")

        pinScreen.enterPin(DEFAULT_PIN)
        transferWalletScreen.createNewWallet()
        assertAll(
            { assertTrue(personalizeSuccessScreen.visible(), "personalize loading screen is not visible") },
            { assertTrue(personalizeSuccessScreen.successMessageVisible(), "success text is not visible") },
            { assertTrue(personalizeSuccessScreen.cardsVisible(), "cards not visible") }
        )

        personalizeSuccessScreen.clickNextButton()
        assertTrue(dashboardScreen.visible(), "dashboard screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC3 Authentication with auth server fails")
    @Tags(Tag("a11yBatch1"))
    fun verifySessionCanceledScreen(testInfo: TestInfo) {
        setUp(testInfo)
        personalizeInformScreen.clickDigidLoginButton()
        digidLoginStartWebPage.switchToWebViewContext()
        digidLoginStartWebPage.clickMockLoginButton()
        digidLoginMockWebPage.enterBsn("123456789")
        digidLoginMockWebPage.clickLoginButton()
        personalizeAuthenticatingWithDigidScreen.switchToNativeContext()
        assertAll(
            { assertTrue(personalizeAuthenticatingWithDigidScreen.loginFailedMessageVisible(), "message is not visible") },
            { assertTrue(personalizeAuthenticatingWithDigidScreen.goToDigiDSiteButtonVisible(), "go to digid site button is not visible") },
            { assertTrue(personalizeAuthenticatingWithDigidScreen.tryAgainButtonVisible(), "try again button is not visible") },
        )
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC4 User rejects issued attributes")
    fun verifyBackButton(testInfo: TestInfo) {
        setUp(testInfo)
        personalizeInformScreen.clickDigidLoginButton()
        digidLoginStartWebPage.switchToWebViewContext()
        digidLoginStartWebPage.clickMockLoginButton()
        digidLoginMockWebPage.login(DEFAULT_BSN)
        personalizePidPreviewScreen.switchToNativeContext()

        PersonalizePidPreviewScreen().clickRejectButton()
        assertTrue(personalizePidDataIncorrectScreen.visible(), "personalize pid data incorrect screen is not visible")

        personalizePidDataIncorrectScreen.clickBottomBackButton()
        personalizePidDataIncorrectScreen.clickBottomPrimaryButton()
        assertTrue(personalizePidPreviewScreen.visible(), "personalize pid preview screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC2 Issuance fails")
    fun pidIssuanceFails(testInfo: TestInfo) {
        setUp(testInfo)
        personalizeInformScreen.clickDigidLoginButton()
        digidLoginStartWebPage.switchToWebViewContext()
        digidLoginStartWebPage.clickMockLoginButton()
        digidLoginMockWebPage.login(DEFAULT_BSN)

        personalizePidPreviewScreen.switchToNativeContext()
        personalizePidPreviewScreen.clickAcceptButton()
        personalizePidPreviewScreen.disableInternetConnection()
        pinScreen.enterPin(DEFAULT_PIN)
        assertAll(
            { assertTrue(noInternetErrorScreen.headlineVisible(), "headline is not visible") },
            { assertTrue(noInternetErrorScreen.descriptionVisible(), "description is not visible") },
            { assertTrue(noInternetErrorScreen.tryAgainButtonVisible(), "try again button is not visible") },
        )
        noInternetErrorScreen.enableInternetConnection()
    }
}
