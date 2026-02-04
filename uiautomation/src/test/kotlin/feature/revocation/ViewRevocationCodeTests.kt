package feature.revocation

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.dashboard.DashboardScreen
import screen.issuance.PersonalizeAuthenticatingWithDigidScreen
import screen.issuance.PersonalizeInformScreen
import screen.issuance.PersonalizePidPreviewScreen
import screen.issuance.PersonalizeSuccessScreen
import screen.issuance.TransferWalletScreen
import screen.menu.MenuScreen
import screen.security.PinScreen
import screen.security.RevocationCodeSettingsScreen
import screen.security.RevocationCodeSetupScreen
import screen.settings.SettingsScreen
import screen.web.digid.DigidLoginMockWebPage
import screen.web.digid.DigidLoginStartWebPage

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("UC9.11 View revocation code")
class ViewRevocationCodeTests : TestBase() {

    private lateinit var pinScreen: PinScreen
    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var personalizeInformScreen: PersonalizeInformScreen
    private lateinit var revocationCodeSetupScreen: RevocationCodeSetupScreen
    private lateinit var revocationCodeSettingScreen: RevocationCodeSettingsScreen
    private lateinit var digidLoginStartWebPage: DigidLoginStartWebPage
    private lateinit var personalizeAuthenticatingWithDigidScreen: PersonalizeAuthenticatingWithDigidScreen
    private lateinit var digidLoginMockWebPage: DigidLoginMockWebPage
    private lateinit var personalizePidPreviewScreen: PersonalizePidPreviewScreen
    private lateinit var transferWalletScreen: TransferWalletScreen
    private lateinit var personalizeSuccessScreen: PersonalizeSuccessScreen
    private lateinit var menuScreen: MenuScreen
    private lateinit var settingsScreen: SettingsScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)

        pinScreen = PinScreen()
        dashboardScreen = DashboardScreen()
        personalizeInformScreen = PersonalizeInformScreen()
        revocationCodeSetupScreen = RevocationCodeSetupScreen()
        digidLoginStartWebPage = DigidLoginStartWebPage()
        personalizeAuthenticatingWithDigidScreen = PersonalizeAuthenticatingWithDigidScreen()
        digidLoginMockWebPage = DigidLoginMockWebPage()
        personalizePidPreviewScreen = PersonalizePidPreviewScreen()
        transferWalletScreen = TransferWalletScreen()
        personalizeSuccessScreen = PersonalizeSuccessScreen()
        menuScreen = MenuScreen()
        settingsScreen = SettingsScreen()
        revocationCodeSettingScreen = RevocationCodeSettingsScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC71 View revocation code in settings")
    fun verifyEeaCardRevocation(testInfo: TestInfo) {
        setUp(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.RevocationCode)
        val revocationCodeFromSetup = revocationCodeSetupScreen.getRevocationCode()
        revocationCodeSetupScreen.confirmReceival()
        personalizeInformScreen.clickDigidLoginButton()

        digidLoginStartWebPage.switchToWebViewContext()
        personalizeAuthenticatingWithDigidScreen.openApp()
        digidLoginStartWebPage.switchToBrowser()
        digidLoginStartWebPage.switchToWebViewContext()
        digidLoginStartWebPage.clickMockLoginButton()
        digidLoginMockWebPage.login(DEFAULT_BSN)

        personalizePidPreviewScreen.switchToNativeContext()
        personalizePidPreviewScreen.clickAcceptButton()
        pinScreen.enterPin(DEFAULT_PIN)
        transferWalletScreen.createNewWallet()
        personalizeSuccessScreen.clickNextButton()
        dashboardScreen.clickMenuButton()
        menuScreen.clickSettingsButton()
        settingsScreen.clickRevocationCodeButton()
        revocationCodeSettingScreen.clickViewButton()
        pinScreen.enterPin(DEFAULT_PIN)
        val revocationCodeFromSettings = revocationCodeSettingScreen.getRevocationCode()
        assertTrue(revocationCodeFromSettings == revocationCodeFromSetup, "Revocation codes don't match")
    }
}
