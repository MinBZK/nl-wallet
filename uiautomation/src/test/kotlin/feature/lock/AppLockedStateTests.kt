package feature.lock

import helper.TestBase
import navigator.MenuNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.menu.MenuScreen
import screen.security.PinScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${AppLockedStateTests.USE_CASE} App locked state [${AppLockedStateTests.JIRA_ID}]")
class AppLockedStateTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 2.3"
        const val JIRA_ID = "PVW-868"
    }

    private lateinit var pinScreen: PinScreen

    @BeforeEach
    fun setUp() {
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickLogoutButton()

        pinScreen = PinScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1 When the app boots it is locked and displays the PIN entry screen. [${JIRA_ID}]")
    fun verifyAppLocked() {
        assertTrue(pinScreen.pinScreenVisible(), "pin screen is not visible")
    }
}
