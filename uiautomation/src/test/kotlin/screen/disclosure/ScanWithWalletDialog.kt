package screen.disclosure

import util.MobileActions

class ScanWithWalletDialog : MobileActions() {

    private val scanWithWalletDialogTitle = find.byText(l10n.getString("scanWithWalletDialogTitle"))
    private val scanWithWalletDialogBody = find.byText(l10n.getString("scanWithWalletDialogBody"))
    private val scanWithWalletDialogScanCta = find.byText(l10n.getString("scanWithWalletDialogScanCta"))

    fun visible() = isElementVisible(scanWithWalletDialogTitle)
    fun scanWithWalletDialogBodyVisible() = isElementVisible(scanWithWalletDialogBody)
    fun scanWithWalletButtonVisible() = isElementVisible(scanWithWalletDialogScanCta)
}
