package feature.personalize

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertAll
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.dashboard.DashboardScreen
import screen.personalize.PersonalizeSuccessScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${PersonalizeSuccessTests.USE_CASE} App confirms PID issuance to user [${PersonalizeSuccessTests.JIRA_ID}]")
class PersonalizeSuccessTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 3.1"
        const val JIRA_ID = "PVW-1039"
    }

    private lateinit var personalizeSuccessScreen: PersonalizeSuccessScreen

    fun setUp() {
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.PersonalizeSuccess)

        personalizeSuccessScreen = PersonalizeSuccessScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1 When PID was issued successfully, the App displays a confirmation to the User. 2 The confirmation includes a success message. 3 The confirmation includes the issued cards (PID + Address): card, title.[$JIRA_ID]")
    fun verifyPersonalizeSuccessScreen() {
        setUp()
        assertAll(
            { assertTrue(personalizeSuccessScreen.visible(), "personalize loading screen is not visible") },
            { assertTrue(personalizeSuccessScreen.successMessageVisible(), "success text is not visible") },
            { assertTrue(personalizeSuccessScreen.cardsVisible(), "cards not visible") }
        )
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.4 The App offers an entrance to enter the wallet which brings the User to the Dashboard. [$JIRA_ID]")
    fun verifyNavigateToDashboardButton() {
        setUp()
        personalizeSuccessScreen.clickNextButton()

        val dashboardScreen = DashboardScreen()
        assertTrue(dashboardScreen.visible(), "dashboard screen is not visible")
    }
}
