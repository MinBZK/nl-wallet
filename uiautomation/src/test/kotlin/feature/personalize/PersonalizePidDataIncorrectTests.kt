package feature.personalize

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.personalize.PersonalizeInformScreen
import screen.personalize.PersonalizePidDataIncorrectScreen
import screen.personalize.PersonalizePidPreviewScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${PersonalizePidDataIncorrectTests.USE_CASE} User reports that PID/address is incorrect [${PersonalizePidDataIncorrectTests.JIRA_ID}]")
class PersonalizePidDataIncorrectTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 3.1"
        const val JIRA_ID = "PVW-1040"
    }

    private lateinit var personalizePidDataIncorrectScreen: PersonalizePidDataIncorrectScreen

    fun setUp() {
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.PersonalizePidPreview)

        PersonalizePidPreviewScreen().clickRejectButton()

        personalizePidDataIncorrectScreen = PersonalizePidDataIncorrectScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1 When the User rejects, the App shows the 'Incorrect data support screen' that informs the User about what to do in case the data are not correct. 2 The App offers a button for the user to go back to the process. [$JIRA_ID]")
    fun verifyBackButton() {
        setUp()
        assertTrue(personalizePidDataIncorrectScreen.visible(), "personalize pid data incorrect screen is not visible")
        personalizePidDataIncorrectScreen.clickBottomBackButton()

        val personalizePidPreviewScreen = PersonalizePidPreviewScreen()
        assertTrue(personalizePidPreviewScreen.visible(), "personalize pid preview screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.3 The App offers a button to not add the data, which returns to the 'login with DigiD screen'. [$JIRA_ID]")
    fun verifyRejectPidPreviewButton() {
        setUp()
        personalizePidDataIncorrectScreen.clickBottomPrimaryButton()

        val personalizeInformScreen = PersonalizeInformScreen()
        assertTrue(personalizeInformScreen.visible(), "personalize inform screen is not visible")
    }
}
