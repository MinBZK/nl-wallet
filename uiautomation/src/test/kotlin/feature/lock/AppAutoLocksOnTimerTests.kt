package feature.lock

import helper.TestBase
import navigator.MenuNavigator
import navigator.OnboardingNavigator
import navigator.screen.MenuNavigatorScreen
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.personalize.PersonalizeInformScreen
import screen.security.InactivityLockWarningNotification
import screen.security.PinScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${AppLockedStateTests.USE_CASE} App auto-locks on timer [${AppLockedStateTests.JIRA_ID}]")
class AppAutoLocksOnTimerTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 2.3"
        const val JIRA_ID = "PVW-871"
    }

    private lateinit var pinScreen: PinScreen
    private lateinit var personalizeInformScreen: PersonalizeInformScreen
    private lateinit var inactivityLockWarningNotification: InactivityLockWarningNotification

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1 X and Y are configurable security parameters. (initial values: X: 5m, Y: 5m) 2 When the app is foregrounded, and it was last active more than X minutes ago based on monotonic clock (i.e. time since boot), the app shows the lock screen [${JIRA_ID}]")
    fun verifyAppLocksAfterInactive(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        //for test x is configured to 30 seconds
        Thread.sleep(31000)
        pinScreen = PinScreen()
        assertTrue(pinScreen.pinScreenVisible(), "pin screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.3 When the app has been inactive (absence of in-app touch activity) for Y minutes, the app locks itself. 4. Pending processes in the app will not be killed [${JIRA_ID}]")
    fun verifyAppLocksAfterBackground(testInfo: TestInfo) {
        setUp(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.PersonalizePidPreview)
        personalizeInformScreen = PersonalizeInformScreen()
        personalizeInformScreen.putAppInBackground(35)
        pinScreen = PinScreen()
        assertTrue(pinScreen.pinScreenVisible(), "pin screen is not visible")
        pinScreen.enterPin("122222")
        assertTrue(personalizeInformScreen.visible(), "personalize inform screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE. Warning notification before inactivity lock appears before app locks itself and offers a Confirm Activity and Lock action. Users can extend the session")
    fun verifyWarningNotificationBeforeLock(testInfo: TestInfo) {
        setUp(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.PersonalizePidPreview)
        personalizeInformScreen = PersonalizeInformScreen()
        Thread.sleep(5000)
        inactivityLockWarningNotification = InactivityLockWarningNotification()
        assertTrue(inactivityLockWarningNotification.visible())
        assertTrue(inactivityLockWarningNotification.confirmButtonVisible())
        inactivityLockWarningNotification.clickConfirmButton()
        assertTrue(!inactivityLockWarningNotification.visible(), "inactivity warning notification is visible")
        Thread.sleep(22000)
        inactivityLockWarningNotification.clickLockButton()
        pinScreen = PinScreen()
        assertTrue(pinScreen.pinScreenVisible(), "pin screen is not visible")
    }
}
