package feature.lock

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junitpioneer.jupiter.RetryingTest
import screen.dashboard.DashboardScreen
import screen.menu.MenuScreen
import screen.security.PinScreen

@DisplayName("UC 9.7 - Log out of the App [PVW-1226]")
class UserLocksWalletTests : TestBase() {

    private lateinit var menuScreen: MenuScreen

    @BeforeEach
    fun setUp() {
        OnboardingNavigator().toScreen(OnboardingScreen.Dashboard)

        DashboardScreen().clickMenuButton()

        menuScreen = MenuScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("1. The User can log out of the app (lock the app) from the app menu.")
    fun verifyLogoutButton() {
        assertTrue(menuScreen.logoutButtonVisible(), "logout button is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("2. When logging out of the app, the PIN entry screen is displayed.")
    fun verifyLockedState() {
        menuScreen.clickLogoutButton()

        val pinScreen = PinScreen()
        assertTrue(pinScreen.pinScreenVisible(), "pin screen is not visible")
    }
}
