package feature.personalize

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertAll
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.personalize.PersonalizeAuthenticatingWithDigidScreen
import screen.web.digid.DigidLoginMockWebPage
import screen.web.digid.DigidLoginStartWebPage

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${PersonalizeAppHandlesDigidAuthenticationTests.USE_CASE} App handles DigiD authentication [${PersonalizeAppHandlesDigidAuthenticationTests.JIRA_ID}]")
class PersonalizeAppHandlesDigidAuthenticationTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 3.1"
        const val JIRA_ID = "PVW-1035"
    }

    private lateinit var digidLoginStartWebPage: DigidLoginStartWebPage
    private lateinit var personalizeAuthenticatingWithDigidScreen: PersonalizeAuthenticatingWithDigidScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.DigidLoginStartWebPage)
        digidLoginStartWebPage = DigidLoginStartWebPage()
        personalizeAuthenticatingWithDigidScreen = PersonalizeAuthenticatingWithDigidScreen()
    }

    /**
     * 1. OIDC and SAML are used for this interaction.
     * >> This requirement hard, if not impossible to be tested in an e2e setup and should be validated during an audit of the app.
     */

    /**
     * 2. The NL Wallet app forwards the User to a universal link that opens the DigiD app if it is installed or to the browser if it is not, specifying requirement for LoA High (except for environments that use DigiD preprod).
     * >> Can only be tested when app2app digid is enabled, which is not the case for the testenv. Should be manually validated on release.
     */

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE. If the DigiD session is terminated or fails, the User returns to the NL Wallet app. The NL Wallet app then informs the User that the DigiD session was terminated/unsuccessful and offers the option to try again or cancel [$JIRA_ID]")
    fun verifySessionCanceledScreen(testInfo: TestInfo) {
        setUp(testInfo)
        digidLoginStartWebPage.clickMockLoginButton()
        val digidLoginMockWebPage = DigidLoginMockWebPage()
        digidLoginMockWebPage.enterBsn("123456789")
        digidLoginMockWebPage.clickLoginButton()
        assertAll(
            { assertTrue(personalizeAuthenticatingWithDigidScreen.loginFailedMessageVisible(), "message is not visible") },
            { assertTrue(personalizeAuthenticatingWithDigidScreen.goToDigiDSiteButtonVisible(), "go to digid site button is not visible") },
            { assertTrue(personalizeAuthenticatingWithDigidScreen.tryAgainButtonVisible(), "try again button is not visible") },
        )
    }

    /**
     * 4. Upon successful DigiD authentication, the User also returns to the NL Wallet app and continues with Feature
     * >> verified in other tests and practically almost every setup of e2e scenarios
     */

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE. hilst DigiD authentication is pending, the NL Wallet app shows a 'holding' screen that explains that the NL Wallet app awaits DigiD authentication. This screen should offer a cancel button to break out of the holding state and then offer the user to retry (or help). If, after having aborted, the DigiD app responds, this response is ignored. [$JIRA_ID]")
    fun verifySessionPendingScreen(testInfo: TestInfo) {
        setUp(testInfo)
        //this puts the wallet app in the background for 1 seconds and then activates its. Other driver method like navigate.back() or activateApp() did not work and/or needed different implementations for Android and IOS
        digidLoginStartWebPage.putAppInBackground(1)
        assertAll(
            { assertTrue(personalizeAuthenticatingWithDigidScreen.awaitingUserAuthTitleVisible(), "title is not visible") },
            { assertTrue(personalizeAuthenticatingWithDigidScreen.digidLoadingStopCtaVisible(), "stop button is not visible") },
        )
    }
}
