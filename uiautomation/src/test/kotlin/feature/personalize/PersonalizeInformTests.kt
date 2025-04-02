package feature.personalize

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.personalize.PersonalizeInformScreen
import screen.web.digid.DigidIdentityCardWebPage
import screen.web.digid.DigidLoginStartWebPage

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${PersonalizeInformTests.USE_CASE} App informs User before personalization [${PersonalizeInformTests.JIRA_ID}]")
class PersonalizeInformTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 3.1"
        const val JIRA_ID = "PVW-1034"
    }

    private lateinit var personalizeInformScreen: PersonalizeInformScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.PersonalizeInform)

        personalizeInformScreen = PersonalizeInformScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1 The app first displays a screen to the user explaining that to personalize their wallet they must log in with the DigiD App that needs to be activated at level High. [$JIRA_ID]")
    fun verifyPersonalizeInformScreen(testInfo: TestInfo) {
        setUp(testInfo)
        assertTrue(personalizeInformScreen.visible(), "personalize inform screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.2 The app offers guidance for when the user has no DigiD: the user is referred to the DigiD website. [$JIRA_ID]")
    fun verifyDigidWebsiteRedirect(testInfo: TestInfo) {
        setUp(testInfo)
        personalizeInformScreen.clickDigidWebsiteButton()
        personalizeInformScreen.switchToWebView()

        val webPage = DigidIdentityCardWebPage()
        assertTrue(webPage.visible(), "digid identity card web page is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.3 The app offers a button for the user to Log in with DigiD. 4 When the user hits 'Log in with DigiD', DigiD authentication starts. [$JIRA_ID]")
    fun verifyDigidLoginRedirect(testInfo: TestInfo) {
        setUp(testInfo)
        personalizeInformScreen.clickDigidLoginButton()

        val digidLoginStartWebPage = DigidLoginStartWebPage()
        assertTrue(digidLoginStartWebPage.visible(), "digid login start web page is not visible")
    }
}
