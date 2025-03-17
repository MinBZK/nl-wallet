package navigator

import navigator.screen.OnboardingNavigatorScreen
import screen.introduction.IntroductionPrivacyScreen
import screen.introduction.IntroductionScreen
import screen.personalize.PersonalizeInformScreen
import screen.personalize.PersonalizePidPreviewScreen
import screen.personalize.PersonalizeSuccessScreen
import screen.security.PinScreen
import screen.security.SecuritySetupCompletedScreen
import screen.web.digid.DigidLoginMockWebPage
import screen.web.digid.DigidLoginStartWebPage

class OnboardingNavigator {

    companion object {
        const val PIN = "122222"
    }

    fun toScreen(screen: OnboardingNavigatorScreen) {
        if (screen > OnboardingNavigatorScreen.Introduction) IntroductionScreen().clickSkipButton()
        if (screen > OnboardingNavigatorScreen.IntroductionPrivacy) IntroductionPrivacyScreen().clickNextButton()
        if (screen > OnboardingNavigatorScreen.SecurityChoosePin) PinScreen().choosePin(PIN)
        if (screen > OnboardingNavigatorScreen.SecurityConfirmPin) PinScreen().confirmPin(PIN)
        if (screen > OnboardingNavigatorScreen.SetupSecurityConfigureBiometrics) PinScreen().skipBiometricsIfConfigurable()
        if (screen > OnboardingNavigatorScreen.SecuritySetupCompleted) SecuritySetupCompletedScreen().clickNextButton()
        if (screen > OnboardingNavigatorScreen.PersonalizeInform) PersonalizeInformScreen().clickDigidLoginButton()
        if (screen > OnboardingNavigatorScreen.DigidLoginStartWebPage) DigidLoginStartWebPage().clickMockLoginButton()
        if (screen > OnboardingNavigatorScreen.DigidLoginMockWebPage) DigidLoginMockWebPage().clickLoginButton()
        if (screen > OnboardingNavigatorScreen.PersonalizePidPreview) PersonalizePidPreviewScreen().clickAcceptButton()
        if (screen > OnboardingNavigatorScreen.PersonalizeConfirmIssuance) PinScreen().enterPin(PIN)
        if (screen > OnboardingNavigatorScreen.PersonalizeSuccess) PersonalizeSuccessScreen().clickNextButton()

        // App now shows the dashboard screen.
    }
}
