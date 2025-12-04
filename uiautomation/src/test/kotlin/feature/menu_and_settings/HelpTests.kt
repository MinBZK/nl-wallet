package feature.menu_and_settings

import helper.TestBase
import navigator.MenuNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.help.ContactScreen
import screen.help.HelpOverviewScreen
import screen.menu.MenuScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("UC 9.6 Get help")
class HelpTests : TestBase() {

    private lateinit var menuScreen: MenuScreen
    private lateinit var helpOverviewScreen: HelpOverviewScreen
    private lateinit var contactScreen: ContactScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        menuScreen = MenuScreen()
        helpOverviewScreen = HelpOverviewScreen()
        contactScreen = ContactScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC31 Get help")
    @Tags(Tag("a11yBatch2"))
    fun verifyHelp(testInfo: TestInfo) {
        setUp(testInfo)
        menuScreen.clickHelpButton()
        assertTrue(helpOverviewScreen.helpButtonsVisible(), "Help buttons are not visible")

        helpOverviewScreen.clickContactButton()
        assertTrue(contactScreen.visible(), "Contact screen not visible")

        contactScreen.clickBottomBackButton()
        helpOverviewScreen.clickBottomBackButton()
        assertTrue(menuScreen.menuListButtonsVisible(), "Menu screen not visible")
    }
}
