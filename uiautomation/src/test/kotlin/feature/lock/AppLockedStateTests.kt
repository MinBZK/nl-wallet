package feature.lock

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junitpioneer.jupiter.RetryingTest
import screen.security.PinScreen

@DisplayName("UC 2.3 - App locked state [PVW-868]")
class AppLockedStateTests : TestBase() {

    private lateinit var pinScreen: PinScreen

    @BeforeEach
    fun setUp() {
        OnboardingNavigator().toScreen(OnboardingScreen.PersonalizeInform)

        restartApp()

        pinScreen = PinScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("1. When the app boots it is locked and displays the PIN entry screen.")
    fun verifyAppLocked() {
        assertTrue(pinScreen.pinScreenVisible(), "pin screen is not visible")
    }
}
