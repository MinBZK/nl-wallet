package feature.revocation

import helper.RevocationHelper
import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestInstance
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.dashboard.DashboardScreen
import screen.issuance.PersonalizeAuthenticatingWithDigidScreen
import screen.issuance.PersonalizeInformScreen
import screen.issuance.PersonalizePidPreviewScreen
import screen.issuance.PersonalizeSuccessScreen
import screen.issuance.TransferWalletScreen
import screen.revocation.RevocationCodeSetupScreen
import screen.revocation.WalletBlockedByRecoveryCodeScreen
import screen.revocation.WalletBlockedByUserScreen
import screen.revocation.WalletBlockedByWalletIdScreen
import screen.revocation.WalletSolutionBlockedScreen
import screen.security.PinScreen
import screen.web.digid.DigidLoginMockWebPage
import screen.web.digid.DigidLoginStartWebPage
import screen.web.revocation_portal.RevocationPortalWebPage
import util.EnvironmentUtil
import util.MobileActions

@TestInstance(TestInstance.Lifecycle.PER_CLASS)
@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("Wallet Revocation")
class RevokeWalletsTests : TestBase() {

    private lateinit var pinScreen: PinScreen
    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var revocationHelper: RevocationHelper
    private lateinit var walletBlockedByUserScreen: WalletBlockedByUserScreen
    private lateinit var walletBlockedByRecoveryIdScreen: WalletBlockedByRecoveryCodeScreen
    private lateinit var walletBlockedByWalletIdScreen: WalletBlockedByWalletIdScreen
    private lateinit var walletSolutionBlockedScreen: WalletSolutionBlockedScreen

    private lateinit var personalizeInformScreen: PersonalizeInformScreen
    private lateinit var revocationCodeScreen: RevocationCodeSetupScreen
    private lateinit var personalizeAuthenticatingWithDigidScreen: PersonalizeAuthenticatingWithDigidScreen
    private lateinit var digidLoginStartWebPage: DigidLoginStartWebPage
    private lateinit var digidLoginMockWebPage: DigidLoginMockWebPage
    private lateinit var personalizePidPreviewScreen: PersonalizePidPreviewScreen
    private lateinit var personalizeSuccessScreen: PersonalizeSuccessScreen
    private lateinit var transferWalletScreen: TransferWalletScreen
    private lateinit var revocationPortalWebPage: RevocationPortalWebPage

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)

        pinScreen = PinScreen()
        dashboardScreen = DashboardScreen()
        revocationHelper = RevocationHelper()
        walletBlockedByUserScreen = WalletBlockedByUserScreen()
        walletBlockedByRecoveryIdScreen = WalletBlockedByRecoveryCodeScreen()
        walletBlockedByWalletIdScreen = WalletBlockedByWalletIdScreen()
        walletSolutionBlockedScreen = WalletSolutionBlockedScreen()
        revocationCodeScreen = RevocationCodeSetupScreen()
        personalizeInformScreen = PersonalizeInformScreen()
        personalizeAuthenticatingWithDigidScreen = PersonalizeAuthenticatingWithDigidScreen()
        digidLoginStartWebPage = DigidLoginStartWebPage()
        digidLoginMockWebPage = DigidLoginMockWebPage()
        personalizePidPreviewScreen = PersonalizePidPreviewScreen()
        personalizeSuccessScreen = PersonalizeSuccessScreen()
        transferWalletScreen = TransferWalletScreen()
        revocationPortalWebPage = RevocationPortalWebPage()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC74 Revoke wallet with revocation code")
    fun verifyUserWalletRevocation(testInfo: TestInfo) {
        setUp(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.RevocationCode)
        val revocationCode = revocationCodeScreen.getRevocationCode()
        revocationCodeScreen.confirmReceive()
        personalizeInformScreen.clickDigidLoginButton()

        digidLoginStartWebPage.switchToWebViewContext()
        personalizeAuthenticatingWithDigidScreen.openApp()
        personalizeAuthenticatingWithDigidScreen.switchToNativeContext()
        digidLoginStartWebPage.switchToBrowser()
        digidLoginStartWebPage.switchToWebViewContext()
        digidLoginStartWebPage.clickMockLoginButton()
        digidLoginMockWebPage.login(DEFAULT_BSN)

        personalizePidPreviewScreen.switchToNativeContext()
        personalizePidPreviewScreen.clickAcceptButton()
        pinScreen.enterPin(DEFAULT_PIN)
        transferWalletScreen.createNewWallet()
        personalizeSuccessScreen.clickNextButton()
        dashboardScreen.closeApp()

        revocationPortalWebPage.switchToBrowser()
        revocationPortalWebPage.switchToWebViewContext()
        revocationPortalWebPage.openLink(EnvironmentUtil.getVar("REVOCATION_PORTAL_URL") + "support/delete?")
        revocationPortalWebPage.revokeWallet(revocationCode)
        assertTrue(revocationPortalWebPage.successMessageVisible(), "Success message is not visible")

        pinScreen.openApp()
        pinScreen.switchToNativeContext()
        pinScreen.enterPin(DEFAULT_PIN)

        assertTrue(walletBlockedByUserScreen.visible(), "Wallet revoked screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC75 Revoke wallet id")
    fun verifyRevokeWalletId(testInfo: TestInfo) {
        setUp(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.Dashboard)
        dashboardScreen.closeApp()
        revocationHelper.revokeAllActiveWallets()

        pinScreen.openApp()
        pinScreen.switchToNativeContext()
        pinScreen.enterPin(DEFAULT_PIN)

        assertTrue(walletBlockedByWalletIdScreen.visible(), "Wallet revoked screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC76 Revoke wallet by recovery id")
    fun verifyWalletRecoveryIDRevocation(testInfo: TestInfo) {
        setUp(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.Dashboard)
        dashboardScreen.closeApp()

        revocationHelper.revokeWalletByRecoveryCode(DEFAULT_RECOVERY_CODE)

        pinScreen.openApp()
        pinScreen.switchToNativeContext()
        pinScreen.enterPin(DEFAULT_PIN)
        assertTrue(walletBlockedByRecoveryIdScreen.visible(), "Wallet revoked screen is not visible")
        dashboardScreen.closeApp()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC77 Revoke wallet solution")
    fun verifyRevokeWalletSolution(testInfo: TestInfo) {
        setUp(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.Dashboard)
        dashboardScreen.closeApp()
        revocationHelper.revokeWalletSolution()
        Thread.sleep(MobileActions.DEFAULT_RESET_SLEEP)

        pinScreen.openApp()
        pinScreen.switchToNativeContext()
        pinScreen.enterPin(DEFAULT_PIN)

        assertTrue(walletSolutionBlockedScreen.visible(), "Wallet revoked screen is not visible")
    }

    @AfterEach
    fun afterEach() {
        revocationHelper.restoreWalletSolution()
        revocationHelper.deleteFromDenyList(DEFAULT_RECOVERY_CODE)
        Thread.sleep(MobileActions.DEFAULT_RESET_SLEEP)
    }
}
