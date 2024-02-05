package navigator

import screen.digid.DigidLoginMockWebPage
import screen.digid.DigidLoginStartWebPage
import screen.introduction.IntroductionConditionsScreen
import screen.introduction.IntroductionExpectationsScreen
import screen.introduction.IntroductionPrivacyScreen
import screen.introduction.IntroductionScreen
import screen.personalize.PersonalizeInformScreen
import screen.personalize.PersonalizePidPreviewScreen
import screen.personalize.PersonalizeSuccessScreen
import screen.security.PinScreen
import screen.security.SetupSecurityCompletedScreen

class OnboardingNavigator {

    private val pin = "122222"

    fun toScreen(screen: OnboardingScreen) {
        if (screen > OnboardingScreen.Introduction) IntroductionScreen().clickSkipButton()
        if (screen > OnboardingScreen.IntroductionExpectations) IntroductionExpectationsScreen().clickNextButton()
        if (screen > OnboardingScreen.IntroductionPrivacy) IntroductionPrivacyScreen().clickNextButton()
        if (screen > OnboardingScreen.IntroductionConditions) IntroductionConditionsScreen().clickNextButton()
        if (screen > OnboardingScreen.Pin) PinScreen().choosePin(pin)
        if (screen > OnboardingScreen.Pin) PinScreen().confirmPin(pin)
        if (screen > OnboardingScreen.SetupSecurityCompleted) SetupSecurityCompletedScreen().clickNextButton()
        if (screen > OnboardingScreen.PersonalizeInform) PersonalizeInformScreen().clickLoginWithDigidButton()
        if (screen > OnboardingScreen.DigidLoginStartWebPage) DigidLoginStartWebPage().clickMockLoginButton()
        if (screen > OnboardingScreen.DigidLoginMockWebPage) DigidLoginMockWebPage().clickLoginButton()
        if (screen > OnboardingScreen.PersonalizePidPreview) PersonalizePidPreviewScreen().clickAcceptButton()
        if (screen > OnboardingScreen.PersonalizePidPreview) PinScreen().enterPin(pin)
        if (screen > OnboardingScreen.PersonalizeSuccess) PersonalizeSuccessScreen().clickNextButton()

        // App now on dashboard screen; handle further steps inside setUp() of test class
    }
}
