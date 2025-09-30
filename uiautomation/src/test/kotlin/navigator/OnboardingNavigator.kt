package navigator

import helper.TestBase.Companion.DEFAULT_BSN
import helper.TestBase.Companion.DEFAULT_PIN
import navigator.screen.OnboardingNavigatorScreen
import screen.introduction.IntroductionPrivacyScreen
import screen.introduction.IntroductionScreen
import screen.issuance.PersonalizeInformScreen
import screen.issuance.PersonalizePidPreviewScreen
import screen.issuance.PersonalizeSuccessScreen
import screen.issuance.TransferWalletScreen
import screen.security.PinScreen
import screen.security.SecuritySetupCompletedScreen
import screen.web.digid.DigidLoginMockWebPage
import screen.web.digid.DigidLoginStartWebPage

class OnboardingNavigator {

    fun toScreen(screen: OnboardingNavigatorScreen, bsn: String = DEFAULT_BSN) {
        if (screen > OnboardingNavigatorScreen.Introduction) IntroductionScreen().clickSkipButton()
        if (screen > OnboardingNavigatorScreen.IntroductionPrivacy) IntroductionPrivacyScreen().clickNextButton()
        if (screen > OnboardingNavigatorScreen.SecurityChoosePin) PinScreen().choosePin(DEFAULT_PIN)
        if (screen > OnboardingNavigatorScreen.SecurityConfirmPin) PinScreen().confirmPin(DEFAULT_PIN)
        if (screen > OnboardingNavigatorScreen.SetupSecurityConfigureBiometrics) PinScreen().skipBiometricsIfConfigurable()
        if (screen > OnboardingNavigatorScreen.SecuritySetupCompleted) SecuritySetupCompletedScreen().clickNextButton()
        if (screen > OnboardingNavigatorScreen.PersonalizeInform) PersonalizeInformScreen().clickDigidLoginButton()
        if (screen > OnboardingNavigatorScreen.DigidLoginStartWebPage) DigidLoginStartWebPage().clickMockLoginButton()
        if (screen > OnboardingNavigatorScreen.DigidLoginMockWebPage) DigidLoginMockWebPage().login(bsn)
        if (screen > OnboardingNavigatorScreen.PersonalizePidPreview) PersonalizePidPreviewScreen().clickAcceptButton()
        if (screen > OnboardingNavigatorScreen.PersonalizeConfirmIssuance) PinScreen().enterPin(DEFAULT_PIN)
        if (screen > OnboardingNavigatorScreen.PersonalizeTransferWallet) TransferWalletScreen().createNewWallet()
        if (screen > OnboardingNavigatorScreen.PersonalizeSuccess) PersonalizeSuccessScreen().clickNextButton()

        // App now shows the dashboard screen.
    }
}
