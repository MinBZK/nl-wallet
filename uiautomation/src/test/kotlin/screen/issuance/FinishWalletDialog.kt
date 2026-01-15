package screen.issuance

import util.MobileActions

class FinishWalletDialog : MobileActions() {

    private val finishWalletDialogTitle = l10n.getString("finishSetupDialogDescription")

    fun visible() = elementWithTextVisible(finishWalletDialogTitle)
}
