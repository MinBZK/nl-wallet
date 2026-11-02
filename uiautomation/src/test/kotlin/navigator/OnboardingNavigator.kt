package navigator

import helper.TestBase.Companion.DEFAULT_BSN
import helper.TestBase.Companion.DEFAULT_PIN
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import screen.dashboard.DashboardScreen
import screen.demo.DemoScreen
import screen.introduction.IntroductionPrivacyScreen
import screen.introduction.IntroductionScreen
import screen.issuance.PersonalizeInformScreen
import screen.issuance.PersonalizePidPreviewScreen
import screen.issuance.PersonalizeSuccessScreen
import screen.issuance.StartTransferWalletScreen
import screen.menu.MenuScreen
import screen.revocation.RevocationCodeSetupScreen
import screen.security.PinScreen
import screen.security.SecuritySetupCompletedScreen
import screen.web.digid.DigidLoginMockWebPage
import util.MobileActions

class OnboardingNavigator {

    fun toScreen(
        screen: OnboardingNavigatorScreen,
        bsn: String = DEFAULT_BSN,
        dismissNotificationsDrawer: Boolean = true,
    ) {
        Thread.sleep(MobileActions.SET_FRAME_SYNC_MAX_WAIT_MILLIS)
        if (screen > OnboardingNavigatorScreen.Demo) DemoScreen().clickContinueButton()
        if (screen > OnboardingNavigatorScreen.Introduction) IntroductionScreen().clickSkipButton()
        if (screen > OnboardingNavigatorScreen.IntroductionPrivacy) IntroductionPrivacyScreen().clickNextButton()
        if (screen > OnboardingNavigatorScreen.SecurityChoosePin) PinScreen().choosePin(DEFAULT_PIN)
        if (screen > OnboardingNavigatorScreen.SecurityConfirmPin) PinScreen().confirmPin(DEFAULT_PIN)
        if (screen > OnboardingNavigatorScreen.SetupSecurityConfigureBiometrics) PinScreen().skipBiometricsIfConfigurable()
        if (screen > OnboardingNavigatorScreen.SecuritySetupCompleted) SecuritySetupCompletedScreen().clickNextButton()
        if (screen > OnboardingNavigatorScreen.RevocationCode) RevocationCodeSetupScreen().confirmReceive()
        if (screen > OnboardingNavigatorScreen.PersonalizeInform) PersonalizeInformScreen().clickDigidLoginButton()
        if (screen > OnboardingNavigatorScreen.DigidLoginMockWebPage) DigidLoginMockWebPage().login(bsn)
        if (screen > OnboardingNavigatorScreen.PersonalizePidPreview) PersonalizePidPreviewScreen().clickAcceptButton()
        if (screen > OnboardingNavigatorScreen.PersonalizeConfirmIssuance) PinScreen().enterPin(DEFAULT_PIN)
        if (screen > OnboardingNavigatorScreen.PersonalizeTransferWallet) StartTransferWalletScreen().createNewWallet()
        if (screen > OnboardingNavigatorScreen.PersonalizeSuccess) PersonalizeSuccessScreen().clickNextButton()

        // App now shows the dashboard screen.
        if (screen == OnboardingNavigatorScreen.Dashboard && dismissNotificationsDrawer) dismissNotificationsDrawer()
    }

    // The app asks for the notification permission once, a few seconds into the second dashboard visit. Get that
    // out of the way here, so the drawer can't cover a later step of a test that happens to linger on the dashboard.
    private fun dismissNotificationsDrawer() {
        revisitDashboard()
        val dashboardScreen = DashboardScreen()
        assertTrue(
            dashboardScreen.notificationsDrawerVisible(NOTIFICATIONS_DRAWER_TIMEOUT_SECONDS),
            "notification permission drawer is not visible on the second dashboard visit",
        )
        dashboardScreen.dismissNotificationsDrawer()
    }

    fun revisitDashboard() {
        // The app samples the dashboard's visibility in intervals, so both the visit and leaving it again
        // only count once the screen has been shown long enough.
        Thread.sleep(MobileActions.SET_FRAME_SYNC_MAX_WAIT_MILLIS)
        DashboardScreen().clickMenuButton()
        Thread.sleep(MobileActions.SCREEN_TRANSITION_MILLIS)
        MenuScreen().clickBottomBackButton()
    }

    companion object {
        const val NOTIFICATIONS_DRAWER_TIMEOUT_SECONDS = 10L
    }
}
