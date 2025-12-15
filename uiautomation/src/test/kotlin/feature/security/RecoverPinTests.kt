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
import screen.issuance.ResetPinDigiDScreen
import screen.menu.MenuScreen
import screen.security.ForgotPinScreen
import screen.security.PinScreen
import screen.security.RecoverPinSuccesScreen
import screen.web.digid.DigidLoginMockWebPage
import screen.web.digid.DigidLoginStartWebPage

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("UC2.3.2 Recover PIN")
class RecoverPinTests : TestBase() {

    private lateinit var pinScreen: PinScreen
    private lateinit var forgotPinScreen: ForgotPinScreen
    private lateinit var menuScreen: MenuScreen
    private lateinit var resetPinDigiDScreen: ResetPinDigiDScreen
    private lateinit var digidLoginStartWebPage: DigidLoginStartWebPage
    private lateinit var digidLoginMockWebPage: DigidLoginMockWebPage
    private lateinit var recoverPinSuccesScreen: RecoverPinSuccesScreen
    private lateinit var dashboardScreen: DashboardScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)

        pinScreen = PinScreen()
        forgotPinScreen = ForgotPinScreen()
        menuScreen = MenuScreen()
        resetPinDigiDScreen = ResetPinDigiDScreen()
        digidLoginStartWebPage = DigidLoginStartWebPage()
        digidLoginMockWebPage = DigidLoginMockWebPage()
        recoverPinSuccesScreen = RecoverPinSuccesScreen()
        dashboardScreen = DashboardScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC41 Recover PIN happy flow")
    @Tags(Tag("a11yBatch1"))
    fun verifyRecoverPinFlow(testInfo: TestInfo) {
        setUp(testInfo)
        menuScreen.clickLogoutButton()
        assertTrue(pinScreen.pinScreenVisible(), "pin screen is not visible")

        pinScreen.clickForgotPinButton()
        assertTrue(forgotPinScreen.visible(), "forgot pin screen is not visible")

        forgotPinScreen.clickResetPinButton()
        assertTrue(resetPinDigiDScreen.visible(), "reset pin DigiD screen is not visible")

        resetPinDigiDScreen.clickDigidLoginButton()
        digidLoginStartWebPage.switchToWebViewContext()
        assertTrue(digidLoginStartWebPage.visible(), "digid login start web page is not visible")

        digidLoginStartWebPage.clickMockLoginButton()
        digidLoginMockWebPage.login(DEFAULT_BSN)
        pinScreen.switchToNativeContext()
        assertTrue(pinScreen.setupPinScreenVisible(), "choose pin screen is not visible")

        pinScreen.enterPin("222223")
        assertTrue(pinScreen.confirmRecoverPinScreenVisible(), "confirm pin screen is not visible")
        pinScreen.enterPin("222223")
        assertTrue(recoverPinSuccesScreen.visible(), "recover pin success screen screen is not visible")

        recoverPinSuccesScreen.clickToOverviewButton()
        assertTrue(dashboardScreen.visible(), "dashboard screen is not visible")

        dashboardScreen.clickMenuButton()
        menuScreen.clickLogoutButton()
        assertTrue(pinScreen.pinScreenVisible(), "pin screen is not visible")

        pinScreen.enterPin(DEFAULT_PIN)
        assertTrue(pinScreen.pinErrorDialogNonFinalRoundInitialAttemptVisible(), "pin error is not visible")

        pinScreen.closePinIncorrectAlertDialog()
        pinScreen.enterPin("222223")
        assertTrue(dashboardScreen.visible(), "dashboard screen is not visible")
    }
}
