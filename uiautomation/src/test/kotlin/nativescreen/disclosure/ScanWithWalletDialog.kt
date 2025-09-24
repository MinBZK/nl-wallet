package nativescreen.disclosure

import util.NativeMobileActions

class ScanWithWalletDialog : NativeMobileActions() {

    private val scanWithWalletDialogTitle = l10n.getString("scanWithWalletDialogTitle")
    private val scanWithWalletDialogBody = l10n.getString("scanWithWalletDialogBody")
    private val scanWithWalletDialogScanCta = l10n.getString("scanWithWalletDialogScanCta").uppercase()

    fun visible() = elementWithTextVisible(scanWithWalletDialogTitle)
    fun scanWithWalletDialogBodyVisible() = elementWithTextVisible(scanWithWalletDialogBody)
    fun scanWithWalletButtonVisible() = elementWithTextVisible(scanWithWalletDialogScanCta)
}
