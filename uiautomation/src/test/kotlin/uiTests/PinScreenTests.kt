package uiTests

import localization.LocalizationHelper
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junitpioneer.jupiter.RetryingTest
import screens.introduction.IntroductionScreens
import screens.pin.PinScreen
import screens.pin.SetupSecurityCompletedScreen
import screens.pin.SetupSecurityConfirmationErrorScreen

class PinScreenTests : TestBase() {

    private lateinit var introductionScreens: IntroductionScreens
    private lateinit var pinScreen: PinScreen
    private lateinit var setupSecurityCompletedScreen: SetupSecurityCompletedScreen
    private lateinit var setupSecurityConfirmationErrorScreen: SetupSecurityConfirmationErrorScreen
    private lateinit var localizationHelper: LocalizationHelper

    private val correctKeyNumber = "123333"
    private val wrongKeyNumber = "123334"

    @BeforeEach
    fun setUp() {
        introductionScreens = IntroductionScreens()
        pinScreen = PinScreen()
        setupSecurityCompletedScreen = SetupSecurityCompletedScreen()
        setupSecurityConfirmationErrorScreen = SetupSecurityConfirmationErrorScreen()
        localizationHelper = LocalizationHelper()
    }

    @RetryingTest(value = 3, name = "{displayName} - #{index}")
    @DisplayName("UC 2.1 - verify easy pin error message")
    @Tags(Tag("smoke"), Tag("android"), Tag("iOS"))
    fun verifyEasyPin() {
        introductionScreens.clickSkipButton()
        introductionScreens.clickNextButton()

        pinScreen.verifyIfPinScreenIsVisible()
        pinScreen.tapKeyNumber("123456")

        Assertions.assertEquals(
            pinScreen.verifySimpleErrorMessageTitleText(),
            localizationHelper.getLocalizedString("setupSecuritySelectPinErrorPageTitle"),
            "expected easy pin error title"
        )
        Assertions.assertEquals(
            pinScreen.verifySimpleErrorMessageDescriptionText(),
            localizationHelper.getLocalizedString("setupSecuritySelectPinErrorPageAscendingOrDescendingDigitsError"),
            "expected easy pin error description"
        )
        pinScreen.tapKeyNumber("000000")

        Assertions.assertEquals(
            pinScreen.verifySimpleErrorMessageTitleText(),
            localizationHelper.getLocalizedString("setupSecuritySelectPinErrorPageTitle"),
            "expected unique digits pin error title"
        )
        Assertions.assertEquals(
            pinScreen.verifySimpleErrorMessageDescriptionText(),
            localizationHelper.getLocalizedString("setupSecuritySelectPinErrorPageTooFewUniqueDigitsError"),
            "expected unique digits pin error description"
        )
    }

    @RetryingTest(value = 3, name = "{displayName} - #{index}")
    @DisplayName("UC 2.1 - verify wrong pin error message")
    @Tags(Tag("smoke"), Tag("android"), Tag("iOS"))
    fun verifyWrongPin() {
        introductionScreens.clickSkipButton()
        introductionScreens.clickNextButton()

        pinScreen.verifyIfPinScreenIsVisible()
        pinScreen.tapKeyNumber(correctKeyNumber)
        pinScreen.tapKeyNumber(wrongKeyNumber)
        Assertions.assertEquals(
            pinScreen.verifyErrorTitleDifferentAccessCode(),
            localizationHelper.getLocalizedString("setupSecurityConfirmationErrorPageTitle"),
            "expected different pin error title"
        )
        Assertions.assertEquals(
            pinScreen.verifyErrorDescriptionDifferentAccessCode(),
            localizationHelper.getLocalizedString("setupSecurityConfirmationErrorPageDescription"),
            "expected different pin error description"
        )

        pinScreen.tapKeyNumber(wrongKeyNumber)
        Assertions.assertEquals(
            setupSecurityConfirmationErrorScreen.verifyErrorConfirmationFatalErrorTitle(),
            localizationHelper.getLocalizedString("setupSecurityConfirmationErrorPageFatalTitle"),
            "expected fail to setup pin error title"
        )
        Assertions.assertEquals(
            setupSecurityConfirmationErrorScreen.verifyErrorConfirmationFatalErrorDescription(),
            localizationHelper.getLocalizedString("setupSecurityConfirmationErrorPageFatalDescription"),
            "expected fail to setup pin error description"
        )

        setupSecurityConfirmationErrorScreen.clickSelectNewCodeButton()
        pinScreen.verifyIfPinScreenIsVisible()?.let {
            Assertions.assertTrue(
                it,
                "setup pin screen is not visible"
            )
        }
    }

    @RetryingTest(value = 3, name = "{displayName} - #{index}")
    @DisplayName("UC 2.1 - verify correct pin error message")
    @Tags(Tag("smoke"), Tag("android"), Tag("iOS"))
    fun verifyCorrectPin() {
        introductionScreens.clickSkipButton()
        introductionScreens.clickNextButton()

        pinScreen.verifyIfPinScreenIsVisible()?.let {
            Assertions.assertTrue(
                it,
                "setup pin screen is not visible"
            )
        }
        pinScreen.tapKeyNumber(correctKeyNumber)
        pinScreen.tapKeyNumber(correctKeyNumber)

        setupSecurityCompletedScreen.verifyIfSetupSecurityCompletedScreenIsVisible()?.let {
            Assertions.assertTrue(
                it,
                "setup security completed screen is not visible"
            )
        }
    }

    @RetryingTest(value = 3, name = "{displayName} - #{index}")
    @DisplayName("UC 2.1 - verify forgot pin")
    @Tags(Tag("smoke"), Tag("android"), Tag("iOS"))
    fun verifyForgotPin() {
        introductionScreens.clickSkipButton()
        introductionScreens.clickNextButton()

        val isPinScreenVisible = pinScreen.verifyIfPinScreenIsVisible() == true
        assertTrue(isPinScreenVisible, "setup pin screen is not visible")

        pinScreen.tapKeyNumber(correctKeyNumber)
        pinScreen.tapKeyNumber(correctKeyNumber)

        val isSetupSecurityCompletedScreenVisible =
            setupSecurityCompletedScreen.verifyIfSetupSecurityCompletedScreenIsVisible() == true
        assertTrue(
            isSetupSecurityCompletedScreenVisible,
            "setup security completed screen is not visible"
        )
    }
}
