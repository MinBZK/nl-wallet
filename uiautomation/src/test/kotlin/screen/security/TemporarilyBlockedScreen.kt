package screen.security

import helper.LocalizationHelper
import util.MobileActions

class TemporarilyBlockedScreen : MobileActions() {

    private val forgotPinButton  = find.byText(l10n.getString("pinTimeoutScreenClearWalletCta"))
    private val deleteWalletButton = find.byText(l10n.getString("pinTimeoutScreenForgotPinCta"))

    fun deleteWalletButtonVisible() = isElementVisible(deleteWalletButton)

    fun forgotPinButtonVisible() = isElementVisible(forgotPinButton)

    fun timeoutDurationLeftVisible(duration: String): Boolean {
        val selector = l10n.getString("pinTimeoutScreenTimeoutCountdown").replace("{timeLeft}", duration + " " + l10n.translate(LocalizationHelper.Translation.SECONDS))
        val element = find.byText(selector)
        return isElementVisible(element);
    }
}
