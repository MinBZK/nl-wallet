package feature.security

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.personalize.PersonalizeInformScreen
import screen.security.SecuritySetupCompletedScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${SecuritySetupCompletedTests.USE_CASE} Wallet creates account, initializes and confirms to user [${SecuritySetupCompletedTests.JIRA_ID}]")
class SecuritySetupCompletedTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 2.1"
        const val JIRA_ID = "PVW-1217"
    }

    private lateinit var securitySetupCompletedScreen: SecuritySetupCompletedScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.SecuritySetupCompleted)

        securitySetupCompletedScreen = SecuritySetupCompletedScreen()
    }

    /**
     * 1. Wallet registers device secrets to ensure wallet cannot be cloned or moved to another device.
     * >> This requirement hard, if not impossible to be tested in an e2e setup and should be validated during an audit of the app.
     */

    /**
     * 2. Wallet registers the new device and user with the wallet provider.
     * >> This requirement hard, if not impossible to be tested in an e2e setup and should be validated during an audit of the app.
     */

    /**
     * 3. Wallet registers such that possession of device and knowledge of PIN are both required to authenticate in future (UCs 2.3 and 2.4).
     * >> This requirement hard, if not impossible to be tested in an e2e setup and should be validated during an audit of the app.
     */

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.4 Wallet confirms setup to user and offers button to start personalization flow. [$JIRA_ID]")
    @Tags(Tag("runonall"))
    fun verifyStartPersonalization(testInfo: TestInfo) {
        setUp(testInfo)
        securitySetupCompletedScreen.clickNextButton()

        val personalizeInformScreen = PersonalizeInformScreen()
        assertTrue(personalizeInformScreen.visible(), "personalize inform screen is not absent")
    }
}
