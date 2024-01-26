package feature.personalize

import helper.TestBase
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junitpioneer.jupiter.RetryingTest
import screen.personalize.PersonalizeAuthenticatingWithDigidScreen
import screen.personalize.PersonalizeInformScreen
import setup.OnboardingNavigator
import setup.Screen

@DisplayName("UC 3.1 - App performs issuance with PID provider [PVW-1036]")
class PersonalizeAuthenticatingWithDigidScreenTests : TestBase() {

    private val onboardingNavigator = OnboardingNavigator()

    private lateinit var personalizeAuthenticatingWithDigidScreen: PersonalizeAuthenticatingWithDigidScreen

    @BeforeEach
    fun setUp() {
        onboardingNavigator.toScreen(Screen.PersonalizeInform)

        PersonalizeInformScreen().clickLoginWithDigidButton(false)

        personalizeAuthenticatingWithDigidScreen = PersonalizeAuthenticatingWithDigidScreen()
    }

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("1. The App displays a loading screen whilst this process is pending.")
    fun verifyPersonalizeAuthenticatingWithDigidScreen() {
        assertTrue(
            personalizeAuthenticatingWithDigidScreen.visible(),
            "personalize authenticating with digid screen is not visible"
        )
    }

    /*@RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("2. The App requests PID from the PID Provider by providing the OIDC access token that resulted from the DigiD login.")
    fun verifyProvidingAccessToken() {
        // This requirement hard, if not impossible to be tested in an e2e setup and should be validated during an audit of the app.
    }*/

    /*@RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("3. The issuance protocol and format are in accordance with the specifications described in PVW-1059.")
    fun verifyIssuanceProtocol() {
        // This requirement hard, if not impossible to be tested in an e2e setup and should be validated during an audit of the app.
    }*/

    /*@RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("4. Go to FEAT 'User confirms/rejects offered PID' PVW-1037")
    fun verifyAcceptPidPreview() {
        // Covered in PersonalizePidPreviewScreenTests
    }*/

    /*@RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("5. Upon user confirmation, the PID Provider issues the PID to the App.")
    fun verifyPidIssuanceSuccess() {
        // Duplicate requirement from PersonalizePidPreviewScreenTests
    }*/

    /*@RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("6. When PID/address issuance fails, the App displays an appropriate message to the User.")
    fun verifyWrongPin() {
        // Manual test: https://SSSS/jira/browse/PVW-1769
    }*/
}
