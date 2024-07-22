package screen.settings

import util.MobileActions

class ClearDataDialog : MobileActions() {

    private val dialogTitle = find.byText(l10n.getString("resetWalletDialogTitle"))
    private val dialogBody = find.byText(l10n.getString("resetWalletDialogBody"))

    private val cancelButton = find.byText(l10n.getString("resetWalletDialogCancelCta"))
    private val confirmButton = find.byText(l10n.getString("resetWalletDialogConfirmCta"))

    fun informVisible() = isElementVisible(dialogTitle) && isElementVisible(dialogBody)

    fun cancelButtonVisible() = isElementVisible(cancelButton)

    fun confirmButtonVisible() = isElementVisible(confirmButton)

    fun clickConfirmButton() = clickElement(confirmButton)
}
