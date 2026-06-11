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
import screen.help.ActivitiesHelpScreen
import screen.help.ContactScreen
import screen.help.HelpAndInfoScreen
import screen.menu.MenuScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("UC 9.6 Get help")
class HelpTests : TestBase() {

    private lateinit var menuScreen: MenuScreen
    private lateinit var helpAndInfoScreen: HelpAndInfoScreen
    private lateinit var contactScreen: ContactScreen
    private lateinit var activitiesHelpScreen: ActivitiesHelpScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        menuScreen = MenuScreen()
        helpAndInfoScreen = HelpAndInfoScreen()
        contactScreen = ContactScreen()
        activitiesHelpScreen = ActivitiesHelpScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC31 Get help")
    @Tags(Tag("a11yBatch2"))
    fun verifyHelp(testInfo: TestInfo) {
        setUp(testInfo)
        menuScreen.clickHelpAndInfoButton()
        assertTrue(helpAndInfoScreen.visible(), "Help buttons are not visible")

        helpAndInfoScreen.clickContactButton()
        assertTrue(contactScreen.visible(), "Contact screen not visible")

        contactScreen.clickBottomBackButton()
        helpAndInfoScreen.clickActivitiesHelpButton()
        activitiesHelpScreen.clickCardActivitiesButton()
        assertTrue(activitiesHelpScreen.helpAndInfoHeadersVisible(), "Help and Info headers are not visible")

        activitiesHelpScreen.clickFirstHelpGroupButton()
        activitiesHelpScreen.clickSomethingElseButton()
        assertTrue(contactScreen.visible(), "Contact screen not visible")

        contactScreen.clickBottomBackButton()
        activitiesHelpScreen.clickBottomBackButton()
        activitiesHelpScreen.clickBottomBackButton()
        helpAndInfoScreen.clickBottomBackButton()
        helpAndInfoScreen.clickBottomBackButton()
        assertTrue(menuScreen.menuListButtonsVisible(), "Menu screen not visible")
    }
}
