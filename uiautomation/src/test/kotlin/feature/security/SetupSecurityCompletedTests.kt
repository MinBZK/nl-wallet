package feature.security

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.OnboardingScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junitpioneer.jupiter.RetryingTest
import screen.personalize.PersonalizeInformScreen
import screen.security.SetupSecurityCompletedScreen

@DisplayName("UC 2.1 - Wallet creates account, initializes and confirms to user [PVW-1217]")
class SetupSecurityCompletedTests : TestBase() {

    private val onboardingNavigator = OnboardingNavigator()

    private lateinit var setupSecurityCompletedScreen: SetupSecurityCompletedScreen

    @BeforeEach
    fun setUp() {
        onboardingNavigator.toScreen(OnboardingScreen.SetupSecurityCompleted)

        setupSecurityCompletedScreen = SetupSecurityCompletedScreen()
    }

    /*@RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("1. Wallet registers device secrets to ensure wallet cannot be cloned or moved to another device.")
    fun verifyNotPossibleFirst() {
        // This requirement hard, if not impossible to be tested in an e2e setup and should be validated during an audit of the app.
    }*/

    /*@RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("2. Wallet registers the new device and user with the wallet provider.")
    fun verifyNotPossibleSecond() {
        // This requirement hard, if not impossible to be tested in an e2e setup and should be validated during an audit of the app.
    }*/

    /*@RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("3. Wallet registers such that possession of device and knowledge of PIN are both required to authenticate in future (UCs 2.3 and 2.4).")
    fun verifyNotPossibleThird() {
        // This requirement hard, if not impossible to be tested in an e2e setup and should be validated during an audit of the app.
    }*/

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("4. Wallet confirms setup to user and offers button to start personalization flow.")
    fun verifyStartPersonalization() {
        setupSecurityCompletedScreen.clickNextButton()

        val personalizeInformScreen = PersonalizeInformScreen()
        assertTrue(personalizeInformScreen.visible(), "personalize inform screen is not absent")
    }
}
