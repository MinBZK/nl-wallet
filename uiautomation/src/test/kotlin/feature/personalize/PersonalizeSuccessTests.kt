package feature.personalize

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.OnboardingScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junitpioneer.jupiter.RetryingTest
import screen.dashboard.DashboardScreen
import screen.personalize.PersonalizeSuccessScreen

@DisplayName("UC 3.1 - App confirms PID issuance to user [PVW-1039]")
class PersonalizeSuccessTests : TestBase() {

    private lateinit var personalizeSuccessScreen: PersonalizeSuccessScreen

    @BeforeEach
    fun setUp() {
        OnboardingNavigator().toScreen(OnboardingScreen.PersonalizeSuccess)

        personalizeSuccessScreen = PersonalizeSuccessScreen()
    }

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("1. When PID was issued successfully, the App displays a confirmation to the User.")
    fun verifyPersonalizeSuccessScreen() {
        assertTrue(personalizeSuccessScreen.visible(), "personalize loading screen is not visible")
    }

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("2. The confirmation includes a success message.")
    fun verifySuccessMessage() {
        assertTrue(personalizeSuccessScreen.successMessageVisible(), "success text is not visible")
    }

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("3. The confirmation includes the issued cards (PID + Address): card, title.")
    fun verifyIssuedCards() {
        assertTrue(personalizeSuccessScreen.cardsVisible(), "cards not visible")
    }

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("4. The App offers an entrance to enter the wallet which brings the User to the Dashboard.")
    fun verifyNavigateToDashboardButton() {
        personalizeSuccessScreen.clickNextButton()

        val dashboardScreen = DashboardScreen()
        assertTrue(dashboardScreen.visible(), "dashboard screen is not visible")
    }
}
