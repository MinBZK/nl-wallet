package uiTests

import localization.LocalizationHelper
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junitpioneer.jupiter.RetryingTest
import screen.introduction.IntroductionScreen
import screen.pin.PinScreen
import screen.pin.SetupSecurityCompletedScreen
import screen.pin.SetupSecurityConfirmationErrorScreen

class PinScreenTests : TestBase() {

    private val correctKeyNumber = "123333"
    private val wrongKeyNumber = "123334"
    private val localizationHelper = LocalizationHelper()

    private lateinit var introductionScreen: IntroductionScreen
    private lateinit var pinScreen: PinScreen
    private lateinit var setupSecurityCompletedScreen: SetupSecurityCompletedScreen
    private lateinit var setupSecurityConfirmationErrorScreen: SetupSecurityConfirmationErrorScreen

    @BeforeEach
    fun setUp() {
        introductionScreen = IntroductionScreen()
        pinScreen = PinScreen()
        setupSecurityCompletedScreen = SetupSecurityCompletedScreen()
        setupSecurityConfirmationErrorScreen = SetupSecurityConfirmationErrorScreen()
    }

    @RetryingTest(value = 3, name = "{displayName} - #{index}")
    @DisplayName("UC 2.1 - Verify easy pin error message")
    @Tags(Tag("smoke"), Tag("android"), Tag("ios"))
    fun verifyEasyPinErrorMessage() {
        introductionScreen.clickSkipButton()
        introductionScreen.clickNextButton()

        assertTrue(
            pinScreen.waitForScreenVisibility(),
            "pin screen is not visible"
        )

        pinScreen.clickKeyNumber("123456")

        assertEquals(
            pinScreen.readSimpleErrorMessageTitleText(),
            localizationHelper.getLocalizedString("setupSecuritySelectPinErrorPageTitle"),
            "expected easy pin error title"
        )
        assertEquals(
            pinScreen.readSimpleErrorMessageDescriptionText(),
            localizationHelper.getLocalizedString("setupSecuritySelectPinErrorPageAscendingOrDescendingDigitsError"),
            "expected easy pin error description"
        )

        pinScreen.clickKeyNumber("000000")

        assertEquals(
            pinScreen.readSimpleErrorMessageTitleText(),
            localizationHelper.getLocalizedString("setupSecuritySelectPinErrorPageTitle"),
            "expected unique digits pin error title"
        )
        assertEquals(
            pinScreen.readSimpleErrorMessageDescriptionText(),
            localizationHelper.getLocalizedString("setupSecuritySelectPinErrorPageTooFewUniqueDigitsError"),
            "expected unique digits pin error description"
        )
    }

    @RetryingTest(value = 3, name = "{displayName} - #{index}")
    @DisplayName("UC 2.1 - Verify wrong pin error message")
    @Tags(Tag("smoke"), Tag("android"), Tag("ios"))
    fun verifyWrongPin() {
        introductionScreen.clickSkipButton()
        introductionScreen.clickNextButton()

        assertTrue(
            pinScreen.waitForScreenVisibility(),
            "pin screen is not visible"
        )

        pinScreen.clickKeyNumber(correctKeyNumber)
        pinScreen.clickKeyNumber(wrongKeyNumber)

        assertEquals(
            pinScreen.readErrorTitleDifferentAccessCodeText(),
            localizationHelper.getLocalizedString("setupSecurityConfirmationErrorPageTitle"),
            "expected different pin error title"
        )
        assertEquals(
            pinScreen.readErrorDescriptionDifferentAccessCodeText(),
            localizationHelper.getLocalizedString("setupSecurityConfirmationErrorPageDescription"),
            "expected different pin error description"
        )

        pinScreen.clickKeyNumber(wrongKeyNumber)

        assertEquals(
            setupSecurityConfirmationErrorScreen.readErrorConfirmationFatalErrorTitleText(),
            localizationHelper.getLocalizedString("setupSecurityConfirmationErrorPageFatalTitle"),
            "expected fail to setup pin error title"
        )
        assertEquals(
            setupSecurityConfirmationErrorScreen.readErrorConfirmationFatalErrorDescriptionText(),
            localizationHelper.getLocalizedString("setupSecurityConfirmationErrorPageFatalDescription"),
            "expected fail to setup pin error description"
        )

        setupSecurityConfirmationErrorScreen.clickSelectNewCodeButton()

        assertTrue(
            pinScreen.waitForScreenVisibility(),
            "setup pin screen is not visible"
        )
    }

    @RetryingTest(value = 3, name = "{displayName} - #{index}")
    @DisplayName("UC 2.1 - Verify correct pin error message")
    @Tags(Tag("smoke"), Tag("android"), Tag("ios"))
    fun verifyCorrectPin() {
        introductionScreen.clickSkipButton()
        introductionScreen.clickNextButton()

        assertTrue(
            pinScreen.waitForScreenVisibility(),
            "setup pin screen is not visible"
        )

        pinScreen.clickKeyNumber(correctKeyNumber)
        pinScreen.clickKeyNumber(correctKeyNumber)

        assertTrue(
            setupSecurityCompletedScreen.waitForScreenVisibility(),
            "setup security completed screen is not visible"
        )
    }
}
