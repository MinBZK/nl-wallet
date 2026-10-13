package screen.security

import util.MobileActions

class TemporarilyBlockedScreen : MobileActions() {

    private val forgotPinButton = l10n.getString("pinTimeoutScreenForgotPinCta")
    private val deleteWalletButton = l10n.getString("pinTimeoutScreenClearWalletCta")

    fun deleteWalletButtonVisible() = elementWithTextVisible(deleteWalletButton)

    fun forgotPinButtonVisible() = elementWithTextVisible(forgotPinButton)

    fun timeoutMessageVisible(): Boolean {
        val message = l10n.getString("pinTimeoutScreenTimeoutCountdown").substringBefore("{timeLeft}").trim()
        return elementContainingTextVisible(message)
    }
}
