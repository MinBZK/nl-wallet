package nativenavigator

import helper.TestBase.Companion.DEFAULT_BSN
import helper.TestBase.Companion.DEFAULT_PIN
import nativenavigator.screen.OnboardingNavigatorScreen
import nativescreen.introduction.IntroductionPrivacyScreen
import nativescreen.introduction.IntroductionScreen
import nativescreen.issuance.PersonalizeInformScreen
import nativescreen.issuance.PersonalizePidPreviewScreen
import nativescreen.issuance.PersonalizeSuccessScreen
import nativescreen.security.PinScreen
import nativescreen.security.SecuritySetupCompletedScreen
import nativescreen.web.digid.DigidLoginMockWebPage
import nativescreen.web.digid.DigidLoginStartWebPage

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
        if (screen > OnboardingNavigatorScreen.PersonalizeSuccess) PersonalizeSuccessScreen().clickNextButton()

        // App now shows the dashboard screen.
    }
}
