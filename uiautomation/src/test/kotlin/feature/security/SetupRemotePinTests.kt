package feature.security

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.about.AboutScreen
import screen.issuance.PersonalizeInformScreen
import screen.security.PinScreen
import screen.security.SecuritySetupCompletedScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("UC2.1 User chooses PIN")
class SetupRemotePinTests : TestBase() {

    private lateinit var pinScreen: PinScreen
    private lateinit var aboutScreen: AboutScreen
    private lateinit var securitySetupCompletedScreen: SecuritySetupCompletedScreen
    private lateinit var personalizeInformScreen: PersonalizeInformScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.SecurityChoosePin)

        pinScreen = PinScreen()
        aboutScreen = AboutScreen()
        securitySetupCompletedScreen = SecuritySetupCompletedScreen()
        personalizeInformScreen = PersonalizeInformScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC66 Setup PIN happy flow")
    fun verifyChoosePinScreenVisible(testInfo: TestInfo) {
        setUp(testInfo)
        assertTrue(pinScreen.setupPinScreenVisible(), "choose pin screen is not visible")
        assertTrue(pinScreen.pinKeyboardVisible(), "pin keyboard is not visible")

        pinScreen.clickAppInfoButton()
        assertTrue(aboutScreen.visible(), "about screen is not visible")
        aboutScreen.goBack()

        val pin = "12222"
        pinScreen.enterPin(pin)
        assertTrue(pinScreen.enteredPinAbsent(pin), "entered pin is not absent")

        pinScreen.enterPin("2")
        assertTrue(pinScreen.confirmPinScreenVisible(), "confirm pin screen is not visible")
        assertTrue(pinScreen.pinKeyboardVisible(), "pin keyboard is not visible")

        pinScreen.enterPin(DEFAULT_PIN)
        pinScreen.skipBiometricsIfConfigurable()
        assertTrue(securitySetupCompletedScreen.visible(), "setup security completed screen is not visible")

        securitySetupCompletedScreen.clickNextButton()
        assertTrue(personalizeInformScreen.visible(), "personalize inform screen is not absent")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC63 PIN entry does not conform to policy")
    @Tags(Tag("a11yBatch2"))
    fun verifyPinTwoUniqueDigitsError(testInfo: TestInfo) {
        setUp(testInfo)

        pinScreen.enterPin("111111")
        assertTrue(
            pinScreen.choosePinErrorTooFewUniqueDigitsVisible(),
            "choose pin error too few unique digits is not visible"
        )

        pinScreen.closeAlertDialog()
        pinScreen.enterPin("123456")
        assertTrue(
            pinScreen.choosePinErrorSequentialDigitsVisible(),
            "choose pin error sequential digits is not visible"
        )

        pinScreen.closeAlertDialog()
        pinScreen.enterPin("987654")
        assertTrue(
            pinScreen.choosePinErrorSequentialDigitsVisible(),
            "choose pin error sequential digits is not visible"
        )
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC61 PIN entries do not match, try again")
    @Tags(Tag("a11yBatch2"))
    fun verifyIncorrectConfirmPin(testInfo: TestInfo) {
        setUp(testInfo)
        pinScreen.enterPin(DEFAULT_PIN)
        assertTrue(pinScreen.confirmPinScreenVisible(), "confirm pin screen is not visible")

        pinScreen.enterPin("211111")
        assertTrue(pinScreen.confirmPinErrorMismatchVisible(), "confirm pin error mismatch is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC62 PIN entries do not match, choose new PIN")
    @Tags(Tag("a11yBatch2"))
    fun verifyRestartChoosePin(testInfo: TestInfo) {
        setUp(testInfo)
        pinScreen.enterPin(DEFAULT_PIN)
        assertTrue(pinScreen.confirmPinScreenVisible(), "confirm pin screen is not visible")

        pinScreen.enterPin("211111")
        pinScreen.closeAlertDialog()
        pinScreen.enterPin("211111")
        assertTrue(pinScreen.confirmPinErrorMismatchFatalVisible(), "confirm pin error fatal mismatch is not visible")

        pinScreen.clickConfirmPinErrorFatalCta()
        assertTrue(pinScreen.setupPinScreenVisible(), "choose pin screen is not visible")
    }
}
