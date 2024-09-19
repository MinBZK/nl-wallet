package feature.lock

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.dashboard.DashboardScreen
import screen.menu.MenuScreen
import screen.security.PinScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${UserLocksWalletTests.USE_CASE} Log out of the App [${UserLocksWalletTests.JIRA_ID}]")
class UserLocksWalletTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 9.7"
        const val JIRA_ID = "PVW-1226"
    }

    private lateinit var menuScreen: MenuScreen

    @BeforeEach
    fun setUp() {
        OnboardingNavigator().toScreen(OnboardingScreen.Dashboard)

        DashboardScreen().clickMenuButton()

        menuScreen = MenuScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1 The User can log out of the app (lock the app) from the app menu. [${JIRA_ID}]")
    fun verifyLogoutButton() {
        assertTrue(menuScreen.logoutButtonVisible(), "logout button is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.2 When logging out of the app, the PIN entry screen is displayed. [${JIRA_ID}]")
    @Tags(Tag("smoke"))
    fun verifyLockedState() {
        menuScreen.clickLogoutButton()

        val pinScreen = PinScreen()
        assertTrue(pinScreen.pinScreenVisible(), "pin screen is not visible")
    }
}
