package feature.security

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
import screen.dashboard.DashboardScreen
import screen.menu.MenuScreen
import screen.security.ChangePinSuccessScreen
import screen.security.PinScreen
import screen.settings.SettingsScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("UC2.6 User changes PIN")
class ChangeRemotePinTests : TestBase() {

    private lateinit var pinScreen: PinScreen
    private lateinit var menuScreen: MenuScreen
    private lateinit var settingsScreen: SettingsScreen
    private lateinit var changePinSuccessScreen: ChangePinSuccessScreen
    private lateinit var dashboardScreen: DashboardScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)

        pinScreen = PinScreen()
        menuScreen = MenuScreen()
        settingsScreen = SettingsScreen()
        changePinSuccessScreen = ChangePinSuccessScreen()
        dashboardScreen = DashboardScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC45 Change PIN")
    @Tags(Tag("a11yBatch3"))
    fun verifyChoosePinScreenVisible(testInfo: TestInfo) {
        setUp(testInfo)
        menuScreen.clickSettingsButton()
        settingsScreen.clickChangePinButton()
        assertTrue(pinScreen.enterCurrentPinTitleVisible(), "Enter current pin screen is not visible")
        pinScreen.captureA11ySnapshot("ChangePinEnterCurrent")

        pinScreen.enterPin(DEFAULT_PIN)
        assertTrue(pinScreen.selectNewPinTitleVisible(), "Select new pin screen is not visible")
        pinScreen.captureA11ySnapshot("ChangePinSelectNew")

        pinScreen.enterPin("222221")
        assertTrue(pinScreen.confirmNewPinTitleVisible(), "Confirm new pin screen is not visible")
        pinScreen.captureA11ySnapshot("ChangePinConfirmNew")

        pinScreen.enterPin("222221")
        assertTrue(changePinSuccessScreen.visible(), "Change pin success screen is not visible")
        changePinSuccessScreen.captureA11ySnapshot("ChangePinSuccess")

        changePinSuccessScreen.toSettings()
        settingsScreen.clickBackButton()
        menuScreen.clickLogoutButton()

        pinScreen.enterPin(DEFAULT_PIN)
        assertTrue(pinScreen.pinErrorDialogNonFinalRoundInitialAttemptVisible(), "pin error is not visible")
        pinScreen.captureA11ySnapshot("PinErrorIncorrect")

        pinScreen.closePinIncorrectAlertDialog()
        pinScreen.enterPin("222221")

        dashboardScreen.dismissNotificationsDrawer()
        assertTrue(dashboardScreen.visible(), "Dashboard is not visible")
    }
}
