package feature.personalize

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.personalize.PersonalizeInformScreen
import screen.personalize.PersonalizeNoDigidScreen
import screen.web.digid.DigidApplyWebPage
import screen.web.digid.DigidLoginStartWebPage

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${PersonalizeInformTests.USE_CASE} App informs User before personalization [${PersonalizeInformTests.JIRA_ID}]")
class PersonalizeInformTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 3.1"
        const val JIRA_ID = "PVW-1034"
    }

    private lateinit var personalizeInformScreen: PersonalizeInformScreen

    @BeforeEach
    fun setUp() {
        OnboardingNavigator().toScreen(OnboardingScreen.PersonalizeInform)

        personalizeInformScreen = PersonalizeInformScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1 The app first displays a screen to the user explaining that to personalize their wallet they must log in with the DigiD App that needs to be activated at level High. [$JIRA_ID]")
    fun verifyPersonalizeInformScreen() {
        assertTrue(personalizeInformScreen.visible(), "personalize inform screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.2 The app offers guidance for when the user has no DigiD: the user is referred to the DigiD website. [$JIRA_ID]")
    fun verifyNoDigidWebsiteRedirect() {
        personalizeInformScreen.clickNoDigidButton()

        val personalizeNoDigidScreen = PersonalizeNoDigidScreen()
        assertTrue(personalizeNoDigidScreen.visible(), "personalize no digid screen is not visible")

        personalizeNoDigidScreen.clickApplyForDigidButton()
        personalizeNoDigidScreen.switchToWebView()

        val digidApplyWebPage = DigidApplyWebPage()
        assertTrue(digidApplyWebPage.visible(), "digid apply web page is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.3 The app offers a button for the user to Log in with DigiD. [$JIRA_ID]")
    fun verifyLoginDigidButton() {
        personalizeInformScreen.loginWithDigidButtonVisible()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.4 When the user hits 'Log in with DigiD', DigiD authentication starts. [$JIRA_ID]")
    fun verifyLoginDigidRedirect() {
        personalizeInformScreen.clickLoginWithDigidButton()

        val digidLoginStartWebPage = DigidLoginStartWebPage()
        assertTrue(digidLoginStartWebPage.visible(), "digid login start web page is not visible")
    }
}
