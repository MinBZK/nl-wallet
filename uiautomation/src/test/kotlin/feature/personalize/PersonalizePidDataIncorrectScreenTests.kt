package feature.personalize

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.OnboardingScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junitpioneer.jupiter.RetryingTest
import screen.personalize.PersonalizeInformScreen
import screen.personalize.PersonalizePidDataIncorrectScreen
import screen.personalize.PersonalizePidPreviewScreen

@DisplayName("UC 3.1 - User reports that PID/address is incorrect [PVW-1040]")
class PersonalizePidDataIncorrectScreenTests : TestBase() {

    private lateinit var personalizePidDataIncorrectScreen: PersonalizePidDataIncorrectScreen

    @BeforeEach
    fun setUp() {
        OnboardingNavigator().toScreen(OnboardingScreen.PersonalizePidPreview)

        PersonalizePidPreviewScreen().clickRejectButton()

        personalizePidDataIncorrectScreen = PersonalizePidDataIncorrectScreen()
    }

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("1. When the User rejects, the App shows the 'Incorrect data support screen' that informs the User about what to do in case the data are not correct.")
    fun verifyPersonalizePidDataIncorrectScreen() {
        assertTrue(personalizePidDataIncorrectScreen.visible(), "personalize pid data incorrect screen is not visible")
    }

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("2. The App offers a button for the user to go back to the process.")
    fun verifyBackButton() {
        personalizePidDataIncorrectScreen.clickBottomBackButton()

        val personalizePidPreviewScreen = PersonalizePidPreviewScreen()
        assertTrue(personalizePidPreviewScreen.visible(), "personalize pid preview screen is not visible")
    }

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("3. The App offers a button to not add the data, which returns to the 'login with DigiD screen'.")
    fun verifyRejectPidPreviewButton() {
        personalizePidDataIncorrectScreen.clickBottomPrimaryButton()

        val personalizeInformScreen = PersonalizeInformScreen()
        assertTrue(personalizeInformScreen.visible(), "personalize inform screen is not visible")
    }
}
