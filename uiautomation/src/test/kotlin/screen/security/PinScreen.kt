package screen.security

import util.MobileActions

class PinScreen : MobileActions() {

    private val pinScreen = find.byValueKey("pinScreen")
    private val choosePinScreen = find.byValueKey("selectPinScreen")
    private val confirmPinScreen = find.byValueKey("confirmPinScreen")

    private val pinKeyboard = find.byValueKey("pinKeyboard")

    private val aboutAppButton = find.byToolTip(l10n.getString("setupSecurityScreenAboutAppTooltip"))
    private val forgotPinButton = find.byText(l10n.getString("pinScreenForgotPinCta"))
    private val confirmPinErrorFatalCta = find.byValueKey("setupSecurityConfirmationErrorPageFatalCta")

    private val selectPinErrorTooFewUniqueDigits =
        find.byText(l10n.getString("setupSecuritySelectPinErrorPageTooFewUniqueDigitsError"))
    private val selectPinErrorSequentialDigits =
        find.byText(l10n.getString("setupSecuritySelectPinErrorPageAscendingOrDescendingDigitsError"))
    private val confirmPinErrorMismatchTitle = find.byText(l10n.getString("setupSecurityConfirmationErrorPageTitle"))
    private val confirmPinErrorMismatchDescription =
        find.byText(l10n.getString("setupSecurityConfirmationErrorPageDescription"))
    private val confirmPinErrorMismatchFatalTitle =
        find.byText(l10n.getString("setupSecurityConfirmationErrorPageFatalTitle"))
    private val confirmPinErrorMismatchFatalDescription =
        find.byText(l10n.getString("setupSecurityConfirmationErrorPageFatalDescription"))

    fun pinScreenVisible() = isElementVisible(pinScreen)

    fun choosePinScreenVisible() = isElementVisible(choosePinScreen)

    fun confirmPinScreenVisible() = isElementVisible(confirmPinScreen)

    fun pinKeyboardVisible() = isElementVisible(pinKeyboard)

    fun enteredPinAbsent(pin: String) = isElementAbsent(find.byText(pin))

    fun choosePinErrorTooFewUniqueDigitsVisible() = isElementVisible(selectPinErrorTooFewUniqueDigits)

    fun choosePinErrorSequentialDigitsVisible() = isElementVisible(selectPinErrorSequentialDigits)

    fun confirmPinErrorMismatchVisible(): Boolean =
        isElementVisible(confirmPinErrorMismatchTitle) && isElementVisible(confirmPinErrorMismatchDescription)

    fun confirmPinErrorMismatchFatalVisible(): Boolean =
        isElementVisible(confirmPinErrorMismatchFatalTitle) && isElementVisible(confirmPinErrorMismatchFatalDescription)

    fun clickAboutAppButton() = clickElement(aboutAppButton)

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
}
