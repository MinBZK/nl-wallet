package setup

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

    fun toScreen(screen: Screen) {
        if (screen > Screen.Introduction) IntroductionScreen().clickSkipButton()
        if (screen > Screen.IntroductionExpectations) IntroductionExpectationsScreen().clickNextButton()
        if (screen > Screen.IntroductionPrivacy) IntroductionPrivacyScreen().clickNextButton()
        if (screen > Screen.IntroductionConditions) IntroductionConditionsScreen().clickNextButton()
        if (screen > Screen.Pin) PinScreen().enterPin(pin)
        if (screen > Screen.Pin) PinScreen().enterPin(pin)
        if (screen > Screen.SetupSecurityCompleted) SetupSecurityCompletedScreen().clickNextButton()
        if (screen > Screen.PersonalizeInform) PersonalizeInformScreen().clickLoginWithDigidButton()
        if (screen > Screen.DigidLoginStartWebPage) DigidLoginStartWebPage().clickMockLoginButton()
        if (screen > Screen.DigidLoginMockWebPage) DigidLoginMockWebPage().clickLoginButton()
        if (screen > Screen.PersonalizePidPreview) PersonalizePidPreviewScreen().clickAcceptButton()
        if (screen > Screen.PersonalizePidPreview) PinScreen().enterPin(pin)
        if (screen > Screen.PersonalizeSuccess) PersonalizeSuccessScreen().clickNextButton()

        // App now on dashboard screen; handle further steps inside setUp() of test class
    }
}
