package feature.personalize

import helper.TestBase
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Test
import screen.introduction.IntroductionConditionsScreen
import screen.introduction.IntroductionExpectationsScreen
import screen.introduction.IntroductionPrivacyScreen
import screen.introduction.IntroductionScreen
import screen.personalize.PersonalizeInformScreen
import screen.personalize.PersonalizeAuthenticatingWithDigidScreen
import screen.security.PinScreen
import screen.security.SetupSecurityCompletedScreen

@DisplayName("UC 3.1 - App performs issuance with PID provider [PVW-1036]")
class PersonalizeAuthenticatingWithDigidScreenTests : TestBase() {

    private val chosenPin = "122222"

    private lateinit var personalizeAuthenticatingWithDigidScreen: PersonalizeAuthenticatingWithDigidScreen

    @BeforeEach
    fun setUp() {
        val introductionScreen = IntroductionScreen()
        val expectationsScreen = IntroductionExpectationsScreen()
        val privacyScreen = IntroductionPrivacyScreen()
        val conditionsScreen = IntroductionConditionsScreen()
        val pinScreen = PinScreen()
        val setupSecurityCompletedScreen = SetupSecurityCompletedScreen()
        val personalizeInformScreen = PersonalizeInformScreen()

        // Start all tests after login with digid button is clicked on personalize inform screen
        introductionScreen.clickSkipButton()
        expectationsScreen.clickNextButton()
        privacyScreen.clickNextButton()
        conditionsScreen.clickNextButton()
        pinScreen.enterPin(chosenPin)
        pinScreen.enterPin(chosenPin)
        setupSecurityCompletedScreen.clickNextButton()
        personalizeInformScreen.clickLoginWithDigidButton()

        personalizeAuthenticatingWithDigidScreen = PersonalizeAuthenticatingWithDigidScreen()
    }

    @Test
    @DisplayName("1. The App displays a loading screen whilst this process is pending.")
    fun verifyPersonalizeAuthenticatingWithDigidScreen() {
        assertTrue(personalizeAuthenticatingWithDigidScreen.visible(), "personalize authenticating with digid screen is not visible")
    }

    //@Test
    @DisplayName("2. The App requests PID from the PID Provider by providing the OIDC access token that resulted from the DigiD login.")
    fun verifyProvidingAccessToken() {
        // This requirement hard, if not impossible to be tested in an e2e setup and should be validated during an audit of the app.
    }

    //@Test
    @DisplayName("3. The issuance protocol and format are in accordance with the specifications described in PVW-1059.")
    fun verifyIssuanceProtocol() {
        // This requirement hard, if not impossible to be tested in an e2e setup and should be validated during an audit of the app.
    }

    //@Test
    @DisplayName("4. Go to FEAT 'User confirms/rejects offered PID' PVW-1037")
    fun verifyAcceptPidPreview() {
        // Covered in PersonalizePidPreviewScreenTests
    }

    //@Test
    @DisplayName("5. Upon user confirmation, the PID Provider issues the PID to the App.")
    fun verifyPidIssuanceSuccess() {
        // Duplicate requirement from PersonalizePidPreviewScreenTests
    }

    //@Test
    @DisplayName("6. When PID/address issuance fails, the App displays an appropriate message to the User.")
    fun verifyWrongPin() {
        // Manual test: https://SSSS/jira/browse/PVW-1769
    }
}
