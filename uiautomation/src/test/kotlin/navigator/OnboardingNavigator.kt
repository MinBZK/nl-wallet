package navigator

import navigator.screen.OnboardingScreen
import screen.introduction.IntroductionConditionsScreen
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

    fun toScreen(screen: OnboardingScreen) {
        if (screen > OnboardingScreen.Introduction) IntroductionScreen().clickSkipButton()
        if (screen > OnboardingScreen.IntroductionPrivacy) IntroductionPrivacyScreen().clickNextButton()
        if (screen > OnboardingScreen.IntroductionConditions) IntroductionConditionsScreen().clickNextButton()
        if (screen > OnboardingScreen.SecurityChoosePin) PinScreen().choosePin(PIN)
        if (screen > OnboardingScreen.SecurityConfirmPin) PinScreen().confirmPin(PIN)
        if (screen > OnboardingScreen.SecuritySetupCompleted) SecuritySetupCompletedScreen().clickNextButton()
        if (screen > OnboardingScreen.PersonalizeInform) PersonalizeInformScreen().clickLoginWithDigidButton()
        if (screen > OnboardingScreen.DigidLoginStartWebPage) DigidLoginStartWebPage().clickMockLoginButton()
        if (screen > OnboardingScreen.DigidLoginMockWebPage) DigidLoginMockWebPage().clickLoginButton()
        if (screen > OnboardingScreen.PersonalizePidPreview) PersonalizePidPreviewScreen().clickAcceptButton()
        if (screen > OnboardingScreen.PersonalizeConfirmIssuance) PinScreen().enterPin(PIN)
        if (screen > OnboardingScreen.PersonalizeSuccess) PersonalizeSuccessScreen().clickNextButton()

        // App now shows the dashboard screen.
    }
}
