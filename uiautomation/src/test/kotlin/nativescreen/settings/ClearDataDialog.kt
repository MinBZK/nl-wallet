package nativescreen.settings

import util.NativeMobileActions

class ClearDataDialog : NativeMobileActions() {

    private val dialogTitle = l10n.getString("resetWalletDialogTitle")
    private val dialogBody = l10n.getString("resetWalletDialogBody")
    private val cancelButton = l10n.getString("resetWalletDialogCancelCta")
    private val confirmButton = l10n.getString("resetWalletDialogConfirmCta")

    fun informVisible() = elementWithTextVisible(dialogTitle) && elementWithTextVisible(dialogBody)

    fun cancelButtonVisible() = elementWithTextVisible(cancelButton)

    fun confirmButtonVisible() = elementWithTextVisible(confirmButton)

    fun clickConfirmButton() = clickElementWithText(confirmButton)
}
