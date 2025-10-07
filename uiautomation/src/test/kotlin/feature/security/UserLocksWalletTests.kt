package feature.security

import helper.TestBase
import navigator.MenuNavigator
import navigator.OnboardingNavigator
import navigator.screen.MenuNavigatorScreen
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.issuance.PersonalizeInformScreen
import screen.menu.MenuScreen
import screen.security.InactivityLockWarningNotification
import screen.security.PinScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("UC 9.7 Log out of the App")
class UserLocksWalletTests : TestBase() {

    private lateinit var menuScreen: MenuScreen
    private lateinit var pinScreen: PinScreen
    private lateinit var personalizeInformScreen: PersonalizeInformScreen
    private lateinit var inactivityLockWarningNotification: InactivityLockWarningNotification

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)

        menuScreen = MenuScreen()
        pinScreen = PinScreen()
        personalizeInformScreen = PersonalizeInformScreen()
        inactivityLockWarningNotification = InactivityLockWarningNotification()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC69 Manual logout from menu")
    @Tags(Tag("smoke"))
    fun verifyLockedState(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        menuScreen.clickLogoutButton()
        assertTrue(pinScreen.pinScreenVisible(), "pin screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC70 Logout due to inactivity")
    fun verifyAppLocksAfterInactive(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        Thread.sleep(301000)
        assertTrue(pinScreen.pinScreenVisible(), "pin screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC71 Logout due to background timeout")
    fun verifyAppLocksAfterBackground(testInfo: TestInfo) {
        setUp(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.PersonalizePidPreview)
        personalizeInformScreen.putAppInBackground(305)
        pinScreen.switchToNativeContext()
        assertTrue(pinScreen.pinScreenVisible(), "pin screen is not visible")
        pinScreen.enterPin(DEFAULT_PIN)
        assertTrue(personalizeInformScreen.visible(), "personalize inform screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC72 User confirms logout on inactivity prompt, LTC73 User dismisses inactivity prompt")
    fun verifyWarningNotificationBeforeLock(testInfo: TestInfo) {
        setUp(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.PersonalizePidPreview)
        Thread.sleep(240000)
        inactivityLockWarningNotification.switchToNativeContext()
        assertTrue(inactivityLockWarningNotification.visible())
        assertTrue(inactivityLockWarningNotification.confirmButtonVisible())
        inactivityLockWarningNotification.clickConfirmButton()
        assertTrue(!inactivityLockWarningNotification.visible(), "inactivity warning notification is visible")
        Thread.sleep(240000)
        inactivityLockWarningNotification.clickLockButton()
        assertTrue(pinScreen.pinScreenVisible(), "pin screen is not  visible")
    }
}
