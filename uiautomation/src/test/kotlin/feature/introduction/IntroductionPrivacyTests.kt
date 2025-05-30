package feature.introduction

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.introduction.IntroductionPrivacyScreen
import screen.privacy.PrivacyPolicyScreen
import screen.security.PinScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${IntroductionPrivacyTests.USE_CASE} App displays privacy statement [${IntroductionPrivacyTests.JIRA_ID}]")
class IntroductionPrivacyTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 1.1"
        const val JIRA_ID = "PVW-1220"
    }

    private lateinit var privacyScreen: IntroductionPrivacyScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.IntroductionPrivacy)

        privacyScreen = IntroductionPrivacyScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1 The App displays the summary of the privacy statement. [${JIRA_ID}]")
    fun verifyPrivacyScreen(testInfo: TestInfo) {
        setUp(testInfo)
        assertTrue(privacyScreen.visible(), "privacy screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.2 The App offers an entrance to the full privacy statement, which is embedded in the app. [${JIRA_ID}]")
    fun verifyPrivacyPolicyButton(testInfo: TestInfo) {
        setUp(testInfo)
        privacyScreen.clickPrivacyButton()

        val privacyPolicyScreen = PrivacyPolicyScreen()
        assertTrue(privacyPolicyScreen.visible(), "privacy policy screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.3 The User can proceed to setup pin. [${JIRA_ID}]")
    fun verifyNextButton(testInfo: TestInfo) {
        setUp(testInfo)
        privacyScreen.clickNextButton()

        val pinScreen = PinScreen()
        assertTrue(pinScreen.choosePinScreenVisible(), "choose pin screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.4 The App offers a return to the previous screen. [${JIRA_ID}]")
    fun verifyBackButton(testInfo: TestInfo) {
        setUp(testInfo)
        privacyScreen.clickBackButton()
        assertTrue(privacyScreen.absent(), "privacy screen is visible")
    }
}
