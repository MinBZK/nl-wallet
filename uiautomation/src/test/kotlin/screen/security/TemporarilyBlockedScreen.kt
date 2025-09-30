package screen.security

import helper.LocalizationHelper
import util.MobileActions

class TemporarilyBlockedScreen : MobileActions() {

    private val forgotPinButton  = l10n.getString("pinTimeoutScreenClearWalletCta")
    private val deleteWalletButton = l10n.getString("pinTimeoutScreenForgotPinCta")

    fun deleteWalletButtonVisible() = elementWithTextVisible(deleteWalletButton)

    fun forgotPinButtonVisible() = elementWithTextVisible(forgotPinButton)

    fun timeoutDurationLeftVisible(duration: String): Boolean {
        val selector = l10n.getString("pinTimeoutScreenTimeoutCountdown").replace("{timeLeft}", duration + " " + l10n.translate(LocalizationHelper.Translation.SECONDS))
        return elementWithTextVisible(selector);
    }
}
