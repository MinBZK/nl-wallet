package feature.personalize

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.OnboardingScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junitpioneer.jupiter.RetryingTest
import screen.personalize.PersonalizeConfirmPinScreen
import screen.personalize.PersonalizePidDataIncorrectScreen
import screen.personalize.PersonalizePidPreviewScreen

@DisplayName("UC 3.1 - User confirms/rejects offered PID [PVW-1037]")
class PersonalizePidPreviewScreenTests : TestBase() {

    private lateinit var personalizePidPreviewScreen: PersonalizePidPreviewScreen

    @BeforeEach
    fun setUp() {
        OnboardingNavigator().toScreen(OnboardingScreen.PersonalizePidPreview)

        personalizePidPreviewScreen = PersonalizePidPreviewScreen()
    }

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("1. When the PID provider offers PID data, the app displays this PID data to the user.")
    fun verifyPersonalizePidPreviewScreen() {
        assertTrue(personalizePidPreviewScreen.visible(), "personalize pid preview screen is not visible")
    }

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("2. The App displays the PID data in a user friendly / human readable format.")
    fun verifyHumanReadablePidPreviewData() {
        assertTrue(personalizePidPreviewScreen.humanReadablePidDataVisible(), "human readable pid data is not visible")
    }

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("3. The App asks the User to check whether the data is correct, and offers two buttons: confirm and reject.")
    fun verifyConfirmationButtons() {
        assertTrue(personalizePidPreviewScreen.confirmButtonsVisible(), "confirm buttons are not visible")
    }

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("4. When the User confirms, the User must enter their PIN (UC2.4 Confirm a Protected action)")
    fun verifyAcceptPidPreview() {
        personalizePidPreviewScreen.clickAcceptButton()

        val personalizeConfirmPinScreen = PersonalizeConfirmPinScreen()
        assertTrue(personalizeConfirmPinScreen.visible(), "personalize confirm pin screen is not visible")
    }

    /*@RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("5. When the User has confirmed with PIN, the App continues with FEAT 'App performs PID/address issuance with PID provider'.")
    fun verifyPidIssuance() {
        // This requirement hard, if not impossible to be tested in an e2e setup.
    }*/

    /*@RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("6. When the User has entered a wrong PIN too many times (according to PIN policy), the App informs the User that the PIN was entered too many times and they should set up a new PIN. This clears the app locally and triggers UC 2.1 Setup PIN.")
    fun verifyWrongPin() {
        // Test this in different scenario.
    }*/

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("7. When the User rejects, the App displays a support screen.")
    fun verifyRejectPidPreview() {
        personalizePidPreviewScreen.clickRejectButton()

        val personalizePidDataIncorrectScreen = PersonalizePidDataIncorrectScreen()
        assertTrue(personalizePidDataIncorrectScreen.visible(), "personalize pid data incorrect is not visible")
    }
}
