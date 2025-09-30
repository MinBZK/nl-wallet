package screen.security

import util.MobileActions

class InactivityLockWarningNotification : MobileActions() {

    private val notification = l10n.getString("idleWarningDialogTitle")

    private val lockButton = l10n.getString("idleWarningDialogLogoutCta").uppercase()
    private val confirmButton =l10n.getString("idleWarningDialogContinueCta").uppercase()

    fun visible() = elementWithTextVisible(notification)

    fun clickLockButton() = clickElementWithText(lockButton)

    fun clickConfirmButton() = clickElementWithText(confirmButton)

    fun confirmButtonVisible() = elementWithTextVisible(confirmButton)
}
