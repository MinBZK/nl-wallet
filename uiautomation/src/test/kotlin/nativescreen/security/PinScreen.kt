package nativescreen.security

import data.TestConfigRepository.Companion.testConfig
import util.NativeMobileActions

class PinScreen : NativeMobileActions() {

    private val setupSecuritySelectPinPageTitle = l10n.getString("setupSecuritySelectPinPageTitle")
    private val pinScreenHeader = l10n.getString("pinScreenHeader")
    private val confirmPinScreen = l10n.getString("setupSecurityConfirmationPageTitle")
    private val personalizeConfirmPinScreen = l10n.getString("walletPersonalizeConfirmPinPageTitle")
    private val backButton = l10n.getString("generalWCAGBack")
    private val appInfoButton = l10n.getString("generalWCAGInfo")
    private val forgotPinButton = l10n.getString("pinScreenForgotPinCta")
    private val confirmPinErrorFatalCta = l10n.getString("pinConfirmationErrorDialogFatalCta")
    private val skipBiometricsCta = l10n.getString("setupBiometricsPageSkipCta")
    private val closeAlertDialogButton = l10n.getString("generalOkCta")
    private val closeIncorrectPinAlertDialogButton = l10n.getString("pinErrorDialogCloseCta")
    private val pinValidationErrorTooFewUniqueDigits = l10n.getString("pinValidationErrorDialogTooFewUniqueDigitsError")
    private val pinValidationErrorSequentialDigits = l10n.getString("pinValidationErrorDialogAscendingOrDescendingDigitsError")
    private val confirmPinErrorMismatchTitle = l10n.getString("pinConfirmationErrorDialogTitle")
    private val confirmPinErrorMismatchDescription = l10n.getString("pinConfirmationErrorDialogDescription")
    private val confirmPinErrorMismatchFatalTitle = l10n.getString("pinConfirmationErrorDialogFatalTitle")
    private val confirmPinErrorMismatchFatalDescription = l10n.getString("pinConfirmationErrorDialogFatalDescription")
    private val pinErrorDialogNonFinalRoundInitialAttempt = l10n.getString("pinErrorDialogNonFinalRoundInitialAttempt")
    private val pinErrorDialogNonFinalRoundFinalAttempt = l10n.getString("pinErrorDialogNonFinalRoundFinalAttempt")

    fun setupPinScreenVisible() = elementWithTextVisible(setupSecuritySelectPinPageTitle)

    fun pinScreenVisible() = elementWithTextVisible(pinScreenHeader)

    fun confirmPinScreenVisible() = elementWithTextVisible(confirmPinScreen)

    fun personalizeConfirmPinScreenVisible() = elementWithTextVisible(personalizeConfirmPinScreen)

    fun pinKeyboardVisible() = elementWithTextVisible("1")

    fun enteredPinAbsent(pin: String) = !elementWithTextVisible(pin)

    fun choosePinErrorTooFewUniqueDigitsVisible() = elementWithTextVisible(pinValidationErrorTooFewUniqueDigits)

    fun choosePinErrorSequentialDigitsVisible() = elementWithTextVisible(pinValidationErrorSequentialDigits)

    fun confirmPinErrorMismatchVisible() =
        elementWithTextVisible(confirmPinErrorMismatchTitle) && elementWithTextVisible(confirmPinErrorMismatchDescription)

    fun confirmPinErrorMismatchFatalVisible() =
        elementWithTextVisible(confirmPinErrorMismatchFatalTitle) && elementWithTextVisible(confirmPinErrorMismatchFatalDescription)

    fun pinErrorDialogNonFinalRoundInitialAttemptVisible() = elementWithTextVisible(pinErrorDialogNonFinalRoundInitialAttempt)

    fun clickBackButton() = clickElementWithText(backButton)

    fun clickAppInfoButton() = clickElementWithText(appInfoButton)

    fun clickForgotPinButton() = clickElementWithText(forgotPinButton)

    fun choosePin(pin: String) = enterPin(pin)

    fun confirmPin(pin: String) {
        elementWithTextVisible(confirmPinScreen)
        enterPin(pin)
    }

    fun enterPin(pin: String) {
        for (digit in pin) {
            clickElementWithText(digit.toString())
        }
    }

    fun clickConfirmPinErrorFatalCta() = clickElementWithText(confirmPinErrorFatalCta)

    fun skipBiometricsIfConfigurable() {
        if (!testConfig.remote) {
            if (elementWithTextVisible(skipBiometricsCta)) {
                clickElementWithText(skipBiometricsCta)
            }
        }
    }

    fun closeAlertDialog() = clickElementWithText(closeAlertDialogButton)

    fun closePinIncorrectAlertDialog() = clickElementWithText(closeIncorrectPinAlertDialogButton)

    fun pinErrorDialogNonFinalRoundNonFinalAttemptVisible(retriesLeft: String): Boolean {
        val selectortext = l10n.getString("pinErrorDialogNonFinalRoundNonFinalAttempt").replace("{count}", retriesLeft)
        return elementWithTextVisible(selectortext);
    }

    fun pinErrorDialogNonFinalRoundFinalAttemptVisible() = elementWithTextVisible(pinErrorDialogNonFinalRoundFinalAttempt)
}
