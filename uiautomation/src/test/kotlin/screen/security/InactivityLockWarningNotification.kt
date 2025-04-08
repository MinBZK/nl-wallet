package screen.security

import util.MobileActions

class InactivityLockWarningNotification : MobileActions() {

    private val notification = find.byText(l10n.getString("idleWarningDialogTitle"))

    private val lockButton = find.byText(l10n.getString("idleWarningDialogLogoutCta").uppercase())
    private val confirmButton = find.byText(l10n.getString("idleWarningDialogContinueCta").uppercase())

    fun visible() = isElementVisible(notification)

    fun clickLockButton() = clickElement(lockButton)

    fun clickConfirmButton() = clickElement(confirmButton)

    fun confirmButtonVisible() = isElementVisible(confirmButton)
}
