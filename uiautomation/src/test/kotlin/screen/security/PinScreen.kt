package screen.security

import data.TestConfigRepository.Companion.testConfig
import util.MobileActions

class PinScreen : MobileActions() {

    private val pinScreen = find.byValueKey("pinScreen")
    private val choosePinScreen = find.byValueKey("selectPinScreen")
    private val confirmPinScreen = find.byValueKey("confirmPinScreen")
    private val personalizeConfirmPinScreen = find.byValueKey("personalizeConfirmPinPage")

    private val pinKeyboardFirstKey = find.byValueKey("keyboardDigitKey#1")

    private val backButton = find.byToolTip(l10n.getString("generalWCAGBack"))
    private val appInfoButton = find.byToolTip(l10n.getString("generalWCAGInfo"))
    private val forgotPinButton = find.byText(l10n.getString("pinScreenForgotPinCta"))
    private val confirmPinErrorFatalCta = find.byText(l10n.getString("pinConfirmationErrorDialogFatalCta"))
    private val skipBiometricsCta = find.byText(l10n.getString("setupBiometricsPageSkipCta"))
    private val closeAlertDialogButton = find.byText(l10n.getString("generalOkCta"))

    private val pinValidationErrorTooFewUniqueDigits =
        find.byText(l10n.getString("pinValidationErrorDialogTooFewUniqueDigitsError"))
    private val pinValidationErrorSequentialDigits =
        find.byText(l10n.getString("pinValidationErrorDialogAscendingOrDescendingDigitsError"))
    private val confirmPinErrorMismatchTitle = find.byText(l10n.getString("pinConfirmationErrorDialogTitle"))
    private val confirmPinErrorMismatchDescription =
        find.byText(l10n.getString("pinConfirmationErrorDialogDescription"))
    private val confirmPinErrorMismatchFatalTitle =
        find.byText(l10n.getString("pinConfirmationErrorDialogFatalTitle"))
    private val confirmPinErrorMismatchFatalDescription =
        find.byText(l10n.getString("pinConfirmationErrorDialogFatalDescription"))

    private val pinErrorDialogNonFinalRoundInitialAttempt =
        find.byText(l10n.getString("pinErrorDialogNonFinalRoundInitialAttempt"))

    fun pinScreenVisible() = isElementVisible(pinScreen)

    fun choosePinScreenVisible() = isElementVisible(choosePinScreen)

    fun confirmPinScreenVisible() = isElementVisible(confirmPinScreen)

    fun personalizeConfirmPinScreenVisible() = isElementVisible(personalizeConfirmPinScreen)

    fun pinKeyboardVisible() = isElementVisible(pinKeyboardFirstKey)

    fun enteredPinAbsent(pin: String) = isElementAbsent(find.byText(pin))

    fun choosePinErrorTooFewUniqueDigitsVisible() = isElementVisible(pinValidationErrorTooFewUniqueDigits)

    fun choosePinErrorSequentialDigitsVisible() = isElementVisible(pinValidationErrorSequentialDigits)

    fun confirmPinErrorMismatchVisible() =
        isElementVisible(confirmPinErrorMismatchTitle) && isElementVisible(confirmPinErrorMismatchDescription)

    fun confirmPinErrorMismatchFatalVisible() =
        isElementVisible(confirmPinErrorMismatchFatalTitle) && isElementVisible(confirmPinErrorMismatchFatalDescription)

    fun pinErrorDialogNonFinalRoundInitialAttemptVisible() = isElementVisible(pinErrorDialogNonFinalRoundInitialAttempt)

    fun clickBackButton() = clickElement(backButton)

    fun clickAppInfoButton() = clickElement(appInfoButton)

    fun clickForgotPinButton() = clickElement(forgotPinButton)

    fun choosePin(pin: String) = enterPin(pin)

    fun confirmPin(pin: String) = enterPin(pin)

    fun enterPin(pin: String) {
        for (digit in pin) {
            val elementKey = "keyboardDigitKey#$digit"
            clickElement(find.byValueKey(elementKey), false)
        }
    }

    fun clickConfirmPinErrorFatalCta() = clickElement(confirmPinErrorFatalCta)

    fun skipBiometricsIfConfigurable() {
        // Poor man's check for biometrics
        if (!testConfig.remote) {
            clickElement(skipBiometricsCta)
        }
    }

    fun closeAlertDialog() = clickElement(closeAlertDialogButton)
}
