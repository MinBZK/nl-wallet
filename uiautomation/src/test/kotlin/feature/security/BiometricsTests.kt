package feature.security

import helper.LocalizationHelper
import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.dashboard.DashboardScreen
import screen.issuance.PersonalizeInformScreen
import screen.issuance.PersonalizePidPreviewScreen
import screen.issuance.PersonalizeSuccessScreen
import screen.issuance.StartTransferWalletScreen
import screen.menu.MenuScreen
import screen.revocation.RevocationCodeSetupScreen
import screen.security.PinScreen
import screen.security.SecuritySetupCompletedScreen
import screen.settings.BiometricsSetupScreen
import screen.settings.SettingsScreen
import screen.web.digid.DigidLoginMockWebPage

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("Biometric unlock and configuration")
class BiometricsTests : TestBase() {

    private lateinit var l10n: LocalizationHelper
    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var menuScreen: MenuScreen
    private lateinit var settingsScreen: SettingsScreen
    private lateinit var pinScreen: PinScreen
    private lateinit var securitySetupCompletedScreen: SecuritySetupCompletedScreen
    private lateinit var personalizeInformScreen: PersonalizeInformScreen
    private lateinit var revocationCodeSetupScreen: RevocationCodeSetupScreen
    private lateinit var digidLoginMockWebPage: DigidLoginMockWebPage
    private lateinit var personalizePidPreviewScreen: PersonalizePidPreviewScreen
    private lateinit var personalizeSuccessScreen: PersonalizeSuccessScreen
    private lateinit var startTransferWalletScreen: StartTransferWalletScreen
    private lateinit var biometricsSetupScreen: BiometricsSetupScreen

    @AfterEach
    fun tearDown() {
        pinScreen.unenrollBiometrics()
    }

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        l10n = LocalizationHelper()
        dashboardScreen = DashboardScreen()
        menuScreen = MenuScreen()
        settingsScreen = SettingsScreen()
        pinScreen = PinScreen()
        securitySetupCompletedScreen = SecuritySetupCompletedScreen()
        personalizeInformScreen = PersonalizeInformScreen()
        revocationCodeSetupScreen = RevocationCodeSetupScreen()
        digidLoginMockWebPage = DigidLoginMockWebPage()
        personalizePidPreviewScreen = PersonalizePidPreviewScreen()
        personalizeSuccessScreen = PersonalizeSuccessScreen()
        startTransferWalletScreen = StartTransferWalletScreen()
        biometricsSetupScreen = BiometricsSetupScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC38 Unlock app with biometric")
    fun verifyBiometricUnlock(testInfo: TestInfo) {
        setUp(testInfo)
        pinScreen.enrollBiometrics()
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.SetupSecurityConfigureBiometrics)
        pinScreen.enableBiometrics()
        securitySetupCompletedScreen.clickNextButton()
        revocationCodeSetupScreen.confirmReceive()

        personalizeInformScreen.clickDigidLoginButton()
        digidLoginMockWebPage.switchToWebViewContext()
        digidLoginMockWebPage.login(DEFAULT_BSN)

        personalizePidPreviewScreen.switchToNativeContext()
        personalizePidPreviewScreen.clickAcceptButton()
        pinScreen.enterPin(DEFAULT_PIN)
        startTransferWalletScreen.createNewWallet()
        personalizeSuccessScreen.clickNextButton()

        dashboardScreen.clickMenuButton()
        menuScreen.clickLogoutButton()
        pinScreen.openBiometricLogin()
        pinScreen.enterBiometric(true)

        assertTrue(dashboardScreen.visible(), "Dashboard is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC59 No biometric enrolled on device")
    fun verifyNoEnrolledBiometric(testInfo: TestInfo) {
        setUp(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.SecurityConfirmPin)
        PinScreen().confirmPin(DEFAULT_PIN)

        pinScreen.skipBiometricsIfConfigurable()
        assertTrue(securitySetupCompletedScreen.visible(), "Biometric setup is visible")

        securitySetupCompletedScreen.clickNextButton()
        revocationCodeSetupScreen.confirmReceive()
        personalizeInformScreen.clickDigidLoginButton()
        digidLoginMockWebPage.switchToWebViewContext()
        digidLoginMockWebPage.login(DEFAULT_BSN)

        personalizePidPreviewScreen.switchToNativeContext()
        personalizePidPreviewScreen.clickAcceptButton()
        pinScreen.enterPin(DEFAULT_PIN)
        startTransferWalletScreen.createNewWallet()
        personalizeSuccessScreen.clickNextButton()

        dashboardScreen.clickMenuButton()
        menuScreen.clickSettingsButton()
        settingsScreen.clickSetupBiometricsButton()
        assertTrue(biometricsSetupScreen.visible(), "Biometric setup is not visible")

        biometricsSetupScreen.toggleBiometricUnlock()
        biometricsSetupScreen.clickBackButton()
        settingsScreen.clickBackButton()
        menuScreen.clickLogoutButton()

        assertTrue(!pinScreen.biometricUnlockAvailable(), "Biometric unlock is available")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC60 Disable biometrics")
    fun verifyDisableBiometrics(testInfo: TestInfo) {
        setUp(testInfo)
        pinScreen.enrollBiometrics()
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.SetupSecurityConfigureBiometrics)
        pinScreen.enableBiometrics()
        securitySetupCompletedScreen.clickNextButton()
        revocationCodeSetupScreen.confirmReceive()

        personalizeInformScreen.clickDigidLoginButton()
        digidLoginMockWebPage.switchToWebViewContext()
        digidLoginMockWebPage.login(DEFAULT_BSN)

        personalizePidPreviewScreen.switchToNativeContext()
        personalizePidPreviewScreen.clickAcceptButton()
        pinScreen.enterPin(DEFAULT_PIN)
        startTransferWalletScreen.createNewWallet()
        personalizeSuccessScreen.clickNextButton()

        dashboardScreen.clickMenuButton()
        menuScreen.clickSettingsButton()
        settingsScreen.clickSetupBiometricsButton()
        assertTrue(biometricsSetupScreen.visible(), "Biometric setup is not visible")

        biometricsSetupScreen.toggleBiometricUnlock()
        biometricsSetupScreen.clickBackButton()
        settingsScreen.clickBackButton()
        menuScreen.clickLogoutButton()

        assertTrue(!pinScreen.biometricUnlockAvailable(), "Biometric unlock is available")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC61 Setup biometrics in settings")
    fun verifyBiometricConfigurationInAppSettings(testInfo: TestInfo) {
        setUp(testInfo)
        pinScreen.enrollBiometrics()
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.SecurityConfirmPin)
        PinScreen().confirmPin(DEFAULT_PIN)

        pinScreen.skipBiometricsIfConfigurable()
        assertTrue(securitySetupCompletedScreen.visible(), "Biometric setup is visible")

        securitySetupCompletedScreen.clickNextButton()
        revocationCodeSetupScreen.confirmReceive()
        personalizeInformScreen.clickDigidLoginButton()
        digidLoginMockWebPage.switchToWebViewContext()
        digidLoginMockWebPage.login(DEFAULT_BSN)

        personalizePidPreviewScreen.switchToNativeContext()
        personalizePidPreviewScreen.clickAcceptButton()
        pinScreen.enterPin(DEFAULT_PIN)
        startTransferWalletScreen.createNewWallet()
        personalizeSuccessScreen.clickNextButton()

        dashboardScreen.clickMenuButton()
        menuScreen.clickSettingsButton()
        settingsScreen.clickSetupBiometricsButton()
        assertTrue(biometricsSetupScreen.visible(), "Biometric setup is not visible")

        biometricsSetupScreen.toggleBiometricUnlock()
        pinScreen.enterPin(DEFAULT_PIN)
        securitySetupCompletedScreen.clickCloseButton()
        biometricsSetupScreen.clickBackButton()
        settingsScreen.clickBackButton()
        menuScreen.clickLogoutButton()

        pinScreen.openBiometricLogin()
        pinScreen.enterBiometric(true)
        assertTrue(dashboardScreen.visible(), "Dashboard is not visible")
    }
}
